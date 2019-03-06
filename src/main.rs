use std::fmt::Debug;
use std::collections::HashMap;

mod turing;


fn print_jointed<T> (iter: impl Iterator<Item=Option<T>>)
where T: Copy + Debug + ToString {
  println!("{}",
           iter
             .map(|a| match a {
               None => "_".to_string(),
               Some(a) => a.to_string()
             })
             .collect::<Vec<String>>()
             .join(" ")
  );
}

type Program <Symbol, State> = HashMap<
  (Option<Symbol>, State),
  turing::Action<Symbol, State>
>;

fn build_simple_program() -> Program<char, char> {
  let mut program = HashMap::new();

  program.insert((Some('a'),'a'), turing::Action {
    transition: turing::Transition::State('b'),
    set: Some('b'),
    movement: turing::Movement::Left
  });

  program.insert((Some('b'), 'b'), turing::Action {
    transition: turing::Transition::State('b'),
    set: None,
    movement: turing::Movement::Right
  });

  program.insert((None,'b'), turing::Action {
    transition: turing::Transition::Halt,
    set: Some('a'),
    movement: turing::Movement::Stay
  });

  program
}

fn simple_program() {

  let program = build_simple_program();
  let mut tape = turing::Tape::new();
  let mut vm = turing::VirtualMachine::new('a');

  tape.set(0, Some('a'));

  let head = 0;
  // print it from left to right
  print_jointed(tape.iter_with(head - 2).take(5));

  while let turing::VirtualMachine::Idle(state) = vm {
    let (new_vm, head) = turing::execute(&vm, program.get(&(tape.value(head), state)).unwrap(), &mut tape, head);
    vm = new_vm;

    print_jointed(tape.iter_with(head - 2).take(5));
  }
}

fn main() {
  simple_program();
}