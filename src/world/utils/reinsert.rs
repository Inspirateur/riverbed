pub trait ReinsertTrait {
    fn reinsert(&mut self, old_i: usize, new_i: usize);
}


impl<T> ReinsertTrait for Vec<T> {
    fn reinsert(&mut self, old_i: usize, new_i: usize) {
        if old_i == new_i {
            return;
        }
        if old_i < new_i {
            return self[old_i..=new_i].rotate_right(1);
        }
        // new_i < old_i
        self[new_i..=old_i].rotate_left(1)
    }
}