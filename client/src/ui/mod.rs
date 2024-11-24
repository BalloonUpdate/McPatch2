pub mod main_ui;
pub mod message_box_ui;

#[allow(dead_code)]
type OneshotSender<T> = tokio::sync::oneshot::Sender<T>;
type OneshotReceiver<T> = tokio::sync::oneshot::Receiver<T>;
type MpscSender<T> = tokio::sync::mpsc::Sender<T>;
type MpscReceiver<T> = tokio::sync::mpsc::Receiver<T>;

