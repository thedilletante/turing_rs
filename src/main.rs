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
where T: Copy + Debug + ToString {
  println!("{}",
           cursor_iterator
             .map(|a| match a.lookup() {
               None => "_".to_string(),
               Some(a) => a.to_string()
             })
             .collect::<Vec<String>>()
             .join(" ")
  );
}


#[derive(Debug)]
enum Move {
  Stay,
  Left,
  Right
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Observed <Symbol, State> {
  symbol: Option<Symbol>,
  state: State
}

#[derive(Debug)]
enum Transition <State> {
  State(State),
  Halt
}

#[derive(Debug)]
struct Action <Symbol, State> {
  set: Option<Symbol>,
  movement: Move,
  transition: Transition<State>
}

type Program <Symbol, State> = HashMap<
  Observed<Symbol, State>,
  Action<Symbol, State>
>;

struct IdleVirtualMachine <State> {
  state: State
}

enum VirtualMachine <State> {
  Idle(IdleVirtualMachine<State>),
  Done
}

impl <State> IdleVirtualMachine <State>
where State: Copy {

  fn new(state: State) -> IdleVirtualMachine <State> {
    IdleVirtualMachine {
      state
    }
  }

  fn apply<Symbol>(&self, cursor: &mut Cursor<Symbol>, action: &Action<Symbol, State>) -> VirtualMachine<State>
  where Symbol: Copy,
        Observed<Symbol, State>: Hash + Eq {

    cursor.set(action.set);
    match action.movement {
      Move::Left => {
        cursor.move_left();
      },
      Move::Right => {
        cursor.move_right();
      },
      _ => ()
    };

    match action.transition {
      Transition::State(new_state) => VirtualMachine::Idle(IdleVirtualMachine::new(new_state)),
      Transition::Halt => VirtualMachine::Done
    }
  }
}

impl <State> VirtualMachine <State>
where State: Copy {
  fn new(initial: State) -> VirtualMachine <State> {
    VirtualMachine::Idle(IdleVirtualMachine{
      state: initial
    })
  }
}


fn build_simple_program() -> Program<char, char> {
  let mut program = HashMap::new();

  program.insert(Observed{
    symbol: Some('a'),
    state: 'a'
  }, Action {
    transition: Transition::State('b'),
    set: Some('b'),
    movement: Move::Left
  });

  program.insert(Observed{
    symbol: Some('b'),
    state: 'b'
  }, Action {
    transition: Transition::State('b'),
    set: None,
    movement: Move::Right
  });

  program.insert(Observed{
    symbol: None,
    state: 'b'
  }, Action {
    transition: Transition::Halt,
    set: Some('a'),
    movement: Move::Stay
  });

  program
}

fn simple_program() {

  let program = build_simple_program();
  let mut cursor = Cursor::on_new_empty_tape();

  cursor.set(Some('a'));
  // print it from left to right
  print_jointed(
    cursor.clone()
      .rev().nth(2).unwrap().take(5)
  );

  let mut machine = VirtualMachine::new('a');

  while let VirtualMachine::Idle(idle) = machine {

    let instruction = program.get(&Observed{
      symbol: cursor.lookup(),
      state: idle.state
    }).unwrap();

    machine = idle.apply(&mut cursor, instruction);

    // print it from left to right
    print_jointed(
      cursor.clone()
        .rev().nth(2).unwrap().take(5)
    );
  }
}

//#[warn(dead_code)]
//fn cursor_test() {
//  let cursor = Cursor::on_new_empty_tape();
//
//  // fill the range
//  cursor.clone()
//    .take(5)
//    .enumerate()
//    .for_each(|(i,mut e)| e.set(Some(i)));
//
//  // make an empty cell in the middle
//  cursor.clone()
//    .nth(2).unwrap()
//    .set(None);
//
//  // print it from left to right
//  print_jointed(
//    cursor.clone()
//      .take(5)
//  );
//
//  // print from right to left
//  print_jointed(
//    cursor.clone()
//      // starting with 4th element (or 5th if you start counting with 1)
//      .nth(4).unwrap()
//      // reverse sequence
//      .rev()
//      // and print only 5 elements
//      .take(5)
//  );
//}

fn main() {


  simple_program();
}