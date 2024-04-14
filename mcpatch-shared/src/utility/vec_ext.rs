/// 按条件删除Vec<T>中的元素，会从末尾往前删以避免元素移动
pub trait VecRemoveIf<T> {
    fn remove_if(&mut self, f: impl Fn(&T) -> bool);
}

impl<T> VecRemoveIf<T> for Vec<T> {
    fn remove_if(&mut self, f: impl Fn(&T) -> bool) {
        for i in (0..self.len()).rev() {
            if f(&self[i]) {
                self.remove(i);
            }
        }
    }
}