use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::collections::HashMap;
use std::hash::Hash;

/// Turing machine implementation assumes existence
/// of infinite tape with cells that represents memory.
///
/// My realization consist of two concepts:
///  - a double linked list, where node represents memory unit in the tape;
///  - a cursor which represents a head under the tape.
/// To support moving the cursor to any distance to any direction
/// values of the list is optional to represent empty element.
/// It means the moves of the cursor is able to modify underlaying list.
///
/// Other implementations are also possible:
///  - two containers (vector/list) that are halves of tape
///    directed into different ways (forward/backward);
///  - whenever you can develop.

type CellLink <T> = Rc<RefCell<Cell<T>>>;

struct Cell <T> {
  value: Option<T>,
  left: Option<CellLink<T>>,
  right: Option<CellLink<T>>
}

impl <T> Cell <T> {

  fn single(value: Option<T>) -> CellLink<T> {
    Cell::link(value, None, None)
  }

  fn left_link(value: Option<T>, right: CellLink<T>) -> CellLink<T> {
    let link = Cell::link(value, None, Some(right.clone()));
    right.borrow_mut().left = Some(link.clone());
    link
  }

  fn right_link(value: Option<T>, left: CellLink<T>) -> CellLink<T> {
    let link = Cell::link(value, Some(left.clone()), None);
    left.borrow_mut().right = Some(link.clone());
    link
  }

  fn link(
    value: Option<T>,
    left: Option<CellLink<T>>,
    right: Option<CellLink<T>>
  ) -> CellLink<T> {
    Rc::new(
      RefCell::new(
        Cell {
          value,
          left,
          right
        }
      )
    )
  }
}

/// Cursor moves under tape that it not owned by it.
/// So cloning cursor means making the snapshot
/// of the position of the head under the tape.
#[derive(Clone)]
struct Cursor <T> {
  current: CellLink<T>
}

/// the value of the cell is returned by value in lookup()
/// thus type must support copying
impl <T> Cursor <T>
where T: Copy {

  fn on_new_empty_tape() -> Cursor<T> {
    Cursor {
      current: Cell::single(None)
    }
  }

  fn lookup(&self) -> Option<T> {
    self.current.borrow().value
  }

  fn set(&mut self, value: Option<T>) {
    self.current.borrow_mut().value = value;
  }

  /// Cell::right_link()/Cell::left_link() implementation borrow mutable
  /// reference of provided link to set itself as appropriate link.
  /// It means we can't use match expression to match the link because it will
  /// be borrowed inside the whole scope of match expression.

  fn move_right(&mut self) -> Option<T> {
    let right_cell = if self.current.borrow().right.is_none() {
      Cell::right_link(None, self.current.clone())
    } else {
      self.current.borrow().right.as_ref().unwrap().clone()
    };

    let ret = self.lookup();
    self.current = right_cell;
    ret
  }

  fn move_left(&mut self) -> Option<T> {
    let left_cell = if self.current.borrow().left.is_none() {
      Cell::left_link(None, self.current.clone())
    } else {
      self.current.borrow().left.as_ref().unwrap().clone()
    };

    let ret = self.lookup();
    self.current = left_cell;
    ret
  }
}

/// Be careful. The iterators for cursor type are infinite.
/// They are just making new empty nodes as making progress at any way.
impl <T> Iterator for Cursor <T>
where T: Copy {
  type Item = Cursor<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = Some(self.clone());
    self.move_right();
    ret
  }
}

impl <T> DoubleEndedIterator for Cursor <T>
where T: Copy {
  fn next_back(&mut self) -> Option<Self::Item> {
    let ret = Some(self.clone());
    self.move_left();
    ret
  }
}

fn print_jointed<T> (cursor_iterator: impl Iterator<Item=Cursor<T>>)
where T: Copy + Debug {
  println!("{}",
           cursor_iterator
             .map(|a| format!("{:?}", a.lookup()))
             .collect::<Vec<String>>()
             .join(", ")
  );
}


enum Move {
  Stay,
  Left,
  Right,
  Halt
}

#[derive(Hash, Eq, PartialEq)]
struct Observed <Symbol, State> {
  symbol: Option<Symbol>,
  state: State
}

struct Action <Symbol, State> {
  transition: State,
  set: Option<Symbol>,
  move_: Move
}

type Program <Symbol, State> = HashMap<Observed<Symbol, State>, Action<Symbol, State>>;

enum VirtualMachine <Symbol, State> {
  Working(Program<Symbol, State>, State),
  Done
}

impl <Symbol, State> VirtualMachine <Symbol, State>
where Symbol: Copy,
      State: Copy,
      Observed<Symbol, State>: Hash + Eq {

  fn new(initial: State) -> VirtualMachine <Symbol, State> {
    VirtualMachine::Working(HashMap::new(), initial)
  }

  fn add(&mut self, observed: Observed<Symbol, State>, action: Action<Symbol, State>) {
    match self {
      VirtualMachine::Done => (),
      VirtualMachine::Working(program, _) => {
        program.insert(observed, action);
      }
    }
  }

  fn apply(&mut self, cursor: &mut Cursor<Symbol>) {

    match self {
      VirtualMachine::Done => (),
      VirtualMachine::Working(program, state) => {
        let ref key = Observed {
          symbol: cursor.lookup(),
          state: *state
        };
        let action = program.get(key).unwrap();

        *state = action.transition;
        cursor.set(action.set);
        match action.move_ {
          Move::Halt => {
            *self = VirtualMachine::Done;
          },
          Move::Left => {
            cursor.move_left();
          },
          Move::Right => {
            cursor.move_right();
          },
          Move::Stay => ()
        }
      }
    }
  }

}

fn simple_program() {

  let mut machine = VirtualMachine::new('a');

  machine.add(Observed{
    symbol: Some(1),
    state: 'a'
  }, Action {
    transition: 'b',
    set: Some(2),
    move_: Move::Left
  });

  machine.add(Observed{
    symbol: Some(2),
    state: 'a'
  }, Action {
    transition: 'b',
    set: None,
    move_: Move::Left
  });

  machine.add(Observed{
    symbol: None,
    state: 'b'
  }, Action {
    transition: 'a',
    set: Some(4),
    move_: Move::Right
  });

  let mut cursor = Cursor::on_new_empty_tape();

  cursor.set(Some(1));
  // print it from left to right
  print_jointed(
    cursor.clone()
      .rev().nth(2).unwrap().take(5)
  );

  machine.apply(&mut cursor);
  // print it from left to right
  print_jointed(
    cursor.clone()
      .rev().nth(2).unwrap().take(5)
  );

  machine.apply(&mut cursor);
  // print it from left to right
  print_jointed(
    cursor.clone()
      .rev().nth(2).unwrap().take(5)
  );

  machine.apply(&mut cursor);
  // print it from left to right
  print_jointed(
    cursor.clone()
      .rev().nth(2).unwrap().take(5)
  );
}


fn main() {
  let cursor = Cursor::on_new_empty_tape();

  // fill the range
  cursor.clone()
    .take(5)
    .enumerate()
    .for_each(|(i,mut e)| e.set(Some(i)));

  // make an empty cell in the middle
  cursor.clone()
    .nth(2).unwrap()
    .set(None);

  // print it from left to right
  print_jointed(
    cursor.clone()
      .take(5)
  );

  // print from right to left
  print_jointed(
    cursor.clone()
      // starting with 4th element (or 5th if you start counting with 1)
      .nth(4).unwrap()
      // reverse sequence
      .rev()
      // and print only 5 elements
      .take(5)
  );

  simple_program();
}