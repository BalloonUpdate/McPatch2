use std::cell::RefCell;
use std::sync::Arc;

use nwd::NwgUi;
use nwg::NativeUi;
use tokio::sync::Mutex;

// type OneshotSender<T> = tokio::sync::oneshot::Sender<T>;
// type OneshotReceiver<T> = tokio::sync::oneshot::Receiver<T>;
type MpscSender<T> = tokio::sync::mpsc::Sender<T>;
type MpscReceiver<T> = tokio::sync::mpsc::Receiver<T>;

pub struct DialogContent {
    pub title: String,
    pub content: String,
    pub yesno: bool,
}

enum UiCommand {
    Exit,
    SetVisible(bool),
    SetTitle(String),
    SetLabel(String),
    SetProgress(u32),
    SetLabelSecondary(String),
    PupopDialog(DialogContent),
}

#[derive(NwgUi)]
pub struct AppWindow {
    #[nwg_control(title: "WindowTitle", flags: "WINDOW", size: (400, 150), center: true, topmost: false)]
    #[nwg_events(OnWindowClose: [AppWindow::try_close_window])]
    window: nwg::Window,

    #[nwg_control(position: (2, 15), size: (396, 24), text: "Label", 
        flags: "VISIBLE|ELIPSIS", h_align: HTextAlign::Center, 
        // background_color: Some([255, 0, 255])
    )]
    label: nwg::Label,

    #[nwg_control(position: (2, 55), size: (396, 24), text: "Label Secondary", 
        flags: "VISIBLE|ELIPSIS", h_align: HTextAlign::Center, 
        // background_color: Some([0, 255, 255])
    )]
    label_secondary: nwg::Label,

    #[nwg_control(position: (35, 110), size: (330, 20), range: 0..1000)]
    progress: nwg::ProgressBar,

    #[nwg_control]
    #[nwg_events(OnNotice: [AppWindow::on_noticed])]
    notice: nwg::Notice,

    commands: RefCell<MpscReceiver<UiCommand>>,
    dialog_result: MpscSender<bool>,
}

impl AppWindow {
    pub fn new() -> (AppWindowCommand, app_window_ui::AppWindowUi) {
        let (dialog_result, receiver) = tokio::sync::mpsc::channel(1000);
        let (sender, commands) = tokio::sync::mpsc::channel(1000);
        
        let data = AppWindow {
            window: Default::default(),
            label: Default::default(),
            label_secondary: Default::default(),
            progress: Default::default(),
            notice: Default::default(),
            commands: RefCell::new(commands),
            dialog_result,
        };

        let win = AppWindow::build_ui(data).unwrap();

        let commander = AppWindowCommand { 
            inner: Arc::new(Mutex::new(AppWindowCommanderInner {
                sender, 
                receiver, 
                notice_sender: win.notice.sender(),
            }))
        };

        (commander, win)
    }

    fn on_noticed(&self) {
        // 在本函数里调用nwg::modal_message()会触发on_noticed()的递归，导致运行时借用检查panic
        // 所以吧poll逻辑单独卸载一个闭包里，最小化运行时借用的范围以避免栈溢出的问题
        let poll_command = || -> Option<UiCommand> {
            let mut receiver = self.commands.borrow_mut();

            match receiver.is_empty() {
                true => None,
                false => receiver.blocking_recv(),
            }
        };
        
        while let Some(cmd) = poll_command() {
            match cmd {
                UiCommand::Exit => {
                    self.window.close();
                },
                UiCommand::SetVisible(visible) => {
                    self.window.set_visible(visible);
                },
                UiCommand::SetTitle(title) => {
                    self.window.set_text(&title);
                },
                UiCommand::SetLabel(label) => {
                    self.label.set_text(&label);
                },
                UiCommand::SetProgress(progress) => {
                    self.progress.set_pos(progress);
                },
                UiCommand::SetLabelSecondary(label) => {
                    self.label_secondary.set_text(&label);
                },
                UiCommand::PupopDialog(dialog) => {
                    let prams = nwg::MessageParams {
                        title: &dialog.title,
                        content: &dialog.content,
                        buttons: match dialog.yesno {
                            true => nwg::MessageButtons::OkCancel,
                            false => nwg::MessageButtons::Ok,
                        },
                        icons: nwg::MessageIcons::Info,
                    };
                
                    let choice = nwg::modal_message(&self.window, &prams);

                    let result = match choice {
                        nwg::MessageChoice::No => false,
                        nwg::MessageChoice::Yes => true,
                        nwg::MessageChoice::Ok => true,
                        _ => false,
                    };

                    self.dialog_result.blocking_send(result).unwrap();
                },
            }
        }
    }
    
    fn try_close_window(&self) {
        nwg::stop_thread_dispatch();
    }
}

struct AppWindowCommanderInner {
    sender: MpscSender<UiCommand>,
    receiver: MpscReceiver<bool>,
    notice_sender: nwg::NoticeSender,
}

#[derive(Clone)]
pub struct AppWindowCommand {
    inner: Arc<Mutex<AppWindowCommanderInner>>,
}

impl AppWindowCommand {
    pub async fn exit(&self) {
        let this = self.inner.lock().await;

        this.sender.send(UiCommand::Exit).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_visible(&self, visible: bool) {
        let this = self.inner.lock().await;
        
        this.sender.send(UiCommand::SetVisible(visible)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_title(&self, title: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(UiCommand::SetTitle(title)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_label(&self, text: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(UiCommand::SetLabel(text)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_progress(&self, value: u32) {
        let this = self.inner.lock().await;
        
        this.sender.send(UiCommand::SetProgress(value)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_label_secondary(&self, text: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(UiCommand::SetLabelSecondary(text)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn popup_dialog(&self, dialog: DialogContent) -> bool {
        let mut this = self.inner.lock().await;
        
        this.sender.send(UiCommand::PupopDialog(dialog)).await.unwrap();
        this.notice_sender.notice();

        this.receiver.recv().await.unwrap()
    }
}
