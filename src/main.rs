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

fn main() {
  let program = build_simple_program();

  let mut tape = turing::Tape::new();
  let mut vm = turing::VirtualMachine::new('a');
  let mut head = 0;

  tape.set(head, Some('a'));
  print_jointed(tape.iter_with(head - 2).take(5));

  while let turing::VirtualMachine::Idle(state) = vm {
    let action = program.get(&(tape.value(head), state)).unwrap();
    let (new_vm, new_head) = turing::execute(turing::VirtualMachine::Idle(state), action, &mut tape, head);
    vm = new_vm;
    head = new_head;
    print_jointed(tape.iter_with(head - 2).take(5));
  }
}