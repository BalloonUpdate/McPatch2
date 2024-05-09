use std::cell::RefCell;

use nwd::NwgUi;
use nwg::NativeUi;

pub struct DialogContent {
    pub title: String,
    pub content: String,
    pub yes: String,
    pub no: Option<String>,
}

enum UiCommand {
    Exit,
    SetTitle(String),
    SetLabel(String),
    SetProgress(u32),
    SetProgressLabel(String),
    PupopDialog(DialogContent),
}

#[derive(NwgUi)]
pub struct AppWindow {
    #[nwg_control(title: "Basic example", flags: "WINDOW|VISIBLE", size: (300, 135), position: (300, 300))]
    #[nwg_events(OnWindowClose: [AppWindow::say_goodbye])]
    window: nwg::Window,

    #[nwg_control(text: "Label", size: (280, 35), position: (10, 10))]
    label: nwg::Label,

    #[nwg_control(position: (10, 10), size: (100, 30))]
    progress: nwg::ProgressBar,

    #[nwg_control(text: "Progress Bar", position: (10, 10), size: (100, 30))]
    progress_label: nwg::Label,

    #[nwg_control]
    #[nwg_events(OnNotice: [AppWindow::on_noticed])]
    notice: nwg::Notice,

    sender: tokio::sync::mpsc::Sender<bool>,
    receiver: RefCell<tokio::sync::mpsc::Receiver<UiCommand>>,
}

impl AppWindow {
    pub fn new() -> (AppWindowCommander, app_window_ui::AppWindowUi) {
        let (sender1, receiver1) = tokio::sync::mpsc::channel(1000);
        let (sender2, receiver2) = tokio::sync::mpsc::channel(1000);

        let w = AppWindow {
            window: Default::default(),
            label: Default::default(),
            progress: Default::default(),
            progress_label: Default::default(),
            notice: Default::default(),
            sender: sender1,
            receiver: RefCell::new(receiver2),
        };

        let win = AppWindow::build_ui(w).unwrap();

        let commander = AppWindowCommander { 
            sender: sender2, 
            receiver: receiver1, 
            notice_sender: win.notice.sender(),
        };

        (commander, win)
    }

    fn on_noticed(&self) {
        let mut receiver = self.receiver.borrow_mut();

        while !receiver.is_empty() {
            let cmd = receiver.blocking_recv().unwrap();

            match cmd {
                UiCommand::Exit => {
                    nwg::stop_thread_dispatch();
                },
                UiCommand::SetTitle(title) => {
                    self.window.set_text(&title);
                },
                UiCommand::SetLabel(label) => {
                    self.label.set_text(&label);
                },
                UiCommand::SetProgress(progress) => {
                    self.progress.set_step(progress);
                },
                UiCommand::SetProgressLabel(plabel) => {
                    self.progress_label.set_text(&plabel);
                },
                UiCommand::PupopDialog(dialog) => {
                    let prams = nwg::MessageParams {
                        title: &dialog.title,
                        content: &dialog.content,
                        buttons: nwg::MessageButtons::YesNo,
                        icons: nwg::MessageIcons::Info,
                    };
                
                    let _choice = nwg::message(&prams);

                    // match choice {
                    //     nwg::MessageChoice::Abort => todo!(),
                    //     nwg::MessageChoice::Cancel => todo!(),
                    //     nwg::MessageChoice::Continue => todo!(),
                    //     nwg::MessageChoice::Ignore => todo!(),
                    //     nwg::MessageChoice::No => todo!(),
                    //     nwg::MessageChoice::Ok => todo!(),
                    //     nwg::MessageChoice::Retry => todo!(),
                    //     nwg::MessageChoice::TryAgain => todo!(),
                    //     nwg::MessageChoice::Yes => todo!(),
                    // }

                    self.sender.blocking_send(true).unwrap();
                },
            }
        }
    }
    
    fn say_goodbye(&self) {
        // nwg::modal_info_message(&self.window, "Goodbye", &format!("Goodbye {}", self.name_edit.text()));
        nwg::stop_thread_dispatch();
    }
}

pub struct AppWindowCommander {
    sender: tokio::sync::mpsc::Sender<UiCommand>,
    receiver: tokio::sync::mpsc::Receiver<bool>,
    notice_sender: nwg::NoticeSender,
}

impl AppWindowCommander {
    pub async fn exit(&self) {
        self.sender.send(UiCommand::Exit).await.unwrap();
        self.notice_sender.notice();
    }

    pub async fn set_title(&self, title: String) {
        self.sender.send(UiCommand::SetTitle(title)).await.unwrap();
        self.notice_sender.notice();
    }

    pub async fn set_label(&self, label: String) {
        self.sender.send(UiCommand::SetLabel(label)).await.unwrap();
        self.notice_sender.notice();
    }

    pub async fn set_progress(&self, value: u32) {
        self.sender.send(UiCommand::SetProgress(value)).await.unwrap();
        self.notice_sender.notice();
    }

    pub async fn set_progress_label(&self, plabel: String) {
        self.sender.send(UiCommand::SetProgressLabel(plabel)).await.unwrap();
        self.notice_sender.notice();
    }

    pub async fn popup_dialog(&mut self, dialog: DialogContent) -> bool {
        self.sender.send(UiCommand::PupopDialog(dialog)).await.unwrap();
        self.notice_sender.notice();

        self.receiver.recv().await.unwrap()
    }
}
