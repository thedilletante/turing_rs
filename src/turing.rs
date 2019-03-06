use std::collections::HashMap;

/// Turing machine abstraction requires existence of
/// infinite tape with cells which represents memory.
/// The user is able to walk through the tape in any
/// direction using head by one step at once.
/// Is it possible to look at the cell's content
/// under the head or to change it.

pub struct Tape <T> {
  cells: HashMap<i64, T>
}

impl <T> Tape <T>
where T: Copy {

  pub fn new() -> Tape<T> {
    Tape {
      cells: HashMap::new()
    }
  }

  pub fn value(&self, head: i64) -> Option<T> {
    self.cells.get(&head).map(|x| *x)
  }

  pub fn set(&mut self, head: i64, value: Option<T>) -> Option<T> {
    match value {
      None => self.cells.remove(&head),
      Some(v) => self.cells.insert(head, v)
    }
  }

}

pub enum Movement {
  Left,
  Right,
  Stay
}

pub enum Transition <S> {
  State(S),
  Halt
}

pub struct Action <T, S> {
  pub set: Option<T>,
  pub movement: Movement,
  pub transition: Transition<S>
}

#[derive(Debug)]
pub enum VirtualMachine <S> {
  Idle(S),
  Done
}

impl <S> VirtualMachine <S>
where S: Copy {

  pub fn new(state: S) -> VirtualMachine<S> {
    VirtualMachine::Idle(state)
  }

}

pub fn execute <T, S> (vm: VirtualMachine<S>, action: &Action<T, S>, tape: &mut Tape<T>, head: i64)
  -> (VirtualMachine<S>, i64)
where T: Copy,
      S: Copy {
  match vm {
    VirtualMachine::Done => (VirtualMachine::Done, head),
    VirtualMachine::Idle(_) => {
      tape.set(head, action.set);

      let new_head = match action.movement {
        Movement::Left => head - 1,
        Movement::Right => head + 1,
        Movement::Stay => head
      };

      let vm_state = match action.transition {
        Transition::Halt => VirtualMachine::Done,
        Transition::State(state) => VirtualMachine::Idle(state)
      };

      (vm_state, new_head)
    }
  }
}

/// The iterator trait is implemented for Tape just for convenience.
pub struct TapeIter <'a, T> {
  tape: &'a Tape<T>,
  head: i64
}

impl <T> Tape <T>
where T: Copy {

  #[allow(dead_code)]
  pub fn iter_with(&self, head: i64) -> TapeIter<T> {
    TapeIter {
      tape: self,
      head
    }
  }

  #[allow(dead_code)]
  pub fn iter(&self) -> TapeIter<T> {
    self.iter_with(0)
  }
}

impl <'a, T> Iterator for TapeIter <'a, T>
where T: Copy {
  type Item = Option<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.tape.value(self.head);
    self.head += 1;
    Some(ret)
  }
}

impl <'a, T> DoubleEndedIterator for TapeIter <'a, T>
where T: Copy {
  fn next_back(&mut self) -> Option<Self::Item> {
    let ret = self.tape.value(self.head);
    self.head -= 1;
    Some(ret)
  }
}

#[test]
fn tape_test() {
  let mut tape = Tape::new();

  for i in -1000..1000 {
    assert_eq!(None, tape.value(i));
  }

  assert_eq!(None, tape.set(0, Some('a')));
  assert_eq!(Some('a'), tape.value(0));

  assert_eq!(Some('a'), tape.set(0, None));
  assert_eq!(None, tape.value(0));
}

#[test]
fn iter_test() {
  let mut tape = Tape::new();
  (0..5).for_each(|x| assert_eq!(None, tape.set(x, Some(x))));

  let mut forward_iter = tape.iter();
  assert_eq!(Some(0), forward_iter.next().unwrap());
  assert_eq!(Some(1), forward_iter.next().unwrap());
  assert_eq!(Some(2), forward_iter.next().unwrap());
  assert_eq!(Some(3), forward_iter.next().unwrap());
  assert_eq!(Some(4), forward_iter.next().unwrap());
  assert_eq!(None, forward_iter.next().unwrap());

  let mut backward_iter = tape.iter_with(5).rev();
  assert_eq!(None, backward_iter.next().unwrap());
  assert_eq!(Some(4), backward_iter.next().unwrap());
  assert_eq!(Some(3), backward_iter.next().unwrap());
  assert_eq!(Some(2), backward_iter.next().unwrap());
  assert_eq!(Some(1), backward_iter.next().unwrap());
  assert_eq!(Some(0), backward_iter.next().unwrap());
  assert_eq!(None, backward_iter.next().unwrap());
}