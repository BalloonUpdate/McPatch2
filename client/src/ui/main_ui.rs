use std::cell::RefCell;
use std::sync::Arc;

use nwd::NwgUi;
use nwg::NativeUi;
use tokio::sync::Mutex;

use crate::ui::MpscReceiver;
use crate::ui::MpscSender;

/// 对话框的内容
pub struct DialogContent {
    /// 标题
    pub title: String,

    /// 内容
    pub content: String,

    /// 是否显示Yes+No双按钮，还是仅显示Yes按钮
    pub yesno: bool,
}

/// UI的交互命令
enum Command {
    /// 关闭UI
    Exit,

    /// 设置窗口可见性
    SetVisible(bool),

    /// 设置窗口标题
    SetTitle(String),

    /// 设置窗口里的主文字
    SetLabel(String),

    /// 设置窗口里的副文字
    SetLabelSecondary(String),

    /// 更新进度条
    SetProgress(u32),

    /// 弹出一个模态对话框
    PupopDialog(DialogContent),
}

/// 应用程序的主窗口，负责大部分信息反馈和交互
#[derive(NwgUi)]
pub struct MainWindow {
    #[nwg_control(size: (400, 150), flags: "WINDOW", center: true, topmost: false)]
    #[nwg_events(OnWindowClose: [MainWindow::close])]
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
    #[nwg_events(OnNotice: [MainWindow::on_noticed])]
    notice: nwg::Notice,

    commands: RefCell<MpscReceiver<Command>>,
    dialog_result: MpscSender<bool>,
}

impl MainWindow {
    pub fn new() -> (MainUiCommand, main_window_ui::MainWindowUi) {
        let (dialog_result, receiver) = tokio::sync::mpsc::channel(1000);
        let (sender, commands) = tokio::sync::mpsc::channel(1000);
        
        let data = Self {
            window: Default::default(),
            label: Default::default(),
            label_secondary: Default::default(),
            progress: Default::default(),
            notice: Default::default(),
            commands: RefCell::new(commands),
            dialog_result,
        };

        let ui = Self::build_ui(data).unwrap();

        let cmd = MainUiCommand { 
            inner: Arc::new(Mutex::new(MainUiCommandInner {
                sender, 
                receiver, 
                notice_sender: ui.notice.sender(),
            }))
        };

        (cmd, ui)
    }

    fn on_noticed(&self) {
        // 在本函数里调用nwg::modal_message()会触发on_noticed()的递归，导致运行时借用检查panic
        // 所以吧poll逻辑单独卸载一个闭包里，最小化运行时借用的范围以避免栈溢出的问题
        let poll_command = || -> Option<Command> {
            let mut receiver = self.commands.borrow_mut();

            match receiver.is_empty() {
                true => None,
                false => receiver.blocking_recv(),
            }
        };
        
        while let Some(cmd) = poll_command() {
            match cmd {
                Command::Exit => {
                    self.close();
                },
                Command::SetVisible(visible) => {
                    self.window.set_visible(visible);
                },
                Command::SetTitle(title) => {
                    self.window.set_text(&title);
                },
                Command::SetLabel(label) => {
                    self.label.set_text(&label);
                },
                Command::SetProgress(progress) => {
                    self.progress.set_pos(progress);
                },
                Command::SetLabelSecondary(label) => {
                    self.label_secondary.set_text(&label);
                },
                Command::PupopDialog(dialog) => {
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
    
    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

struct MainUiCommandInner {
    /// 向窗口发送命令的对象
    sender: MpscSender<Command>,

    /// 接收窗口返回的对话框的用户选择，看看用户点击了Yes还是No按钮
    receiver: MpscReceiver<bool>,

    /// 通知窗口有新的命令到达了，需要进行处理
    notice_sender: nwg::NoticeSender,
}

/// 主窗口向外暴露的命令对象，通过channel来和窗口进行交互
#[derive(Clone)]
pub struct MainUiCommand {
    inner: Arc<Mutex<MainUiCommandInner>>,
}

impl MainUiCommand {
    pub async fn exit(&self) {
        let this = self.inner.lock().await;

        this.sender.send(Command::Exit).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_visible(&self, visible: bool) {
        let this = self.inner.lock().await;
        
        this.sender.send(Command::SetVisible(visible)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_title(&self, title: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(Command::SetTitle(title)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_label(&self, text: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(Command::SetLabel(text)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_progress(&self, value: u32) {
        let this = self.inner.lock().await;
        
        this.sender.send(Command::SetProgress(value)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn set_label_secondary(&self, text: String) {
        let this = self.inner.lock().await;
        
        this.sender.send(Command::SetLabelSecondary(text)).await.unwrap();
        this.notice_sender.notice();
    }

    pub async fn popup_dialog(&self, dialog: DialogContent) -> bool {
        let mut this = self.inner.lock().await;
        
        this.sender.send(Command::PupopDialog(dialog)).await.unwrap();
        this.notice_sender.notice();

        this.receiver.recv().await.unwrap()
    }
}
