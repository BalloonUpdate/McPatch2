/// 按条件删除Vec<T>中的元素，会从末尾往前删以避免元素移动。并返回是否有元素被删除
pub trait VecRemoveIf<T> {
    fn remove_if(&mut self, f: impl FnMut(&T) -> bool) -> bool;
}

impl<T> VecRemoveIf<T> for Vec<T> {
    /// 按条件删除元素，如果要删除某个元素请返回true，反之返回false
    fn remove_if(&mut self, mut f: impl FnMut(&T) -> bool) -> bool {
        let mut removed = false;

        for i in (0..self.len()).rev() {
            if f(&self[i]) {
                self.remove(i);
                removed = true;
            }
        }

        removed
    }
}