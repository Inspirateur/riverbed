use std::{collections::{VecDeque, HashSet}, hash::Hash};

pub struct UniqDeque<T: PartialEq + Eq + Hash + Clone> {
	deque: VecDeque<T>,
	set: HashSet<T>
}

impl<T: PartialEq + Eq + Hash + Clone> UniqDeque<T> {
	pub fn new() -> Self {
		Self {
			deque: VecDeque::new(),
			set: HashSet::new()
		}
	}

	pub fn push_front(&mut self, elem: T) {
		if self.set.insert(elem.clone()) {
			self.deque.push_front(elem);
		}
	}

	pub fn push_back(&mut self, elem: T) {
		if self.set.insert(elem.clone()) {
			self.deque.push_back(elem);
		}
	}

	pub fn pop_front(&mut self) -> Option<T> {
		let res = self.deque.pop_front()?;
		self.set.remove(&res);
		Some(res)
	}

	pub fn remove(&mut self, elem: &T) {
		if self.set.remove(elem) {
			let i = self.deque.iter().position(|other| other == elem).unwrap();
			self.deque.remove(i);
		}
	}
}