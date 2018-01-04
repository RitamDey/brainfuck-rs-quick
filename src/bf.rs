use std::mem;
use std::str::Bytes;
use std::u8;



/// The size of the memory.
const MEM_SIZE: usize = 30_000;



/// Interpret a Brainfuck program from a string.
/// Return the result string.
pub fn bf(prog: &str) -> String {
    // Create application memory, and define an output vector
    let mut memory = Memory::new();
    let mut output: Vec<u8> = vec![];

    // Interpret the program, and execute it
    Interpreter::interpret(&mut prog.bytes())
        .execute(&mut memory, &mut output);

    // Parse and output the string
    String::from_utf8(output).unwrap()
}



/// Brainfuck interpreter.
///
/// This interpreter translates a stream of brainfuck program bytes into
/// operations.
struct Interpreter;

impl Interpreter {
    /// Interpret a brainfuck program from the given byte stream.
    /// Output a routine containing the whole state.
    pub fn interpret(program: &mut Bytes) -> Op {
        Interpreter::interpret_routine(program, false)
    }

    /// Interpret the given stream of bytes into a routine.
    /// This routine may be simple, or it may be conditional with makes the routine
    /// loopable.
    ///
    /// The byte stream should be given to `bytes`.
    ///
    /// If `cond` is `true`, this routine is loopable, `false` if it isn't.
    fn interpret_routine(bytes: &mut Bytes, cond: bool) -> Op {
        // Interpret the contained routine operations
        let ops = Interpreter::interpret_vec(bytes);

        // Optimize a zeroing loop
        if cond && ops.iter().all(|op| match *op {
                Op::Inc { .. } => true,
                _ => false,
            }) {
            return Op::Zero;
        }

        // Wrap the oprations in a routine
        Op::Routine {
            ops,
            cond,
        }
    }

    /// Interpret the given stream of bytes into a vector of operations.
    ///
    /// This function returns the vector if the input stream is empty,
    /// or if a loop-end operator has been reached.
    ///
    /// The byte stream should be given to `bytes`.
    fn interpret_vec(bytes: &mut Bytes) -> Vec<Op> {
        // Create an operations vector, and a workspace for the last operation
        // being worked on
        let mut ops = vec![];
        let mut workspace = None;

        // Interpret all bytes until we break
        loop {
            // Find the next byte to process, or break if the stream is emtpy
            let byte = if let Some(byte) = bytes.next() {
                byte
            } else {
                break;
            };

            // Process the byte
            match byte {
                // Seek up
                b'>' => Interpreter::process_workspace_seek(
                    &mut workspace,
                    &mut ops,
                    1,
                ),

                // Seek down
                b'<' => Interpreter::process_workspace_seek(
                    &mut workspace,
                    &mut ops,
                    -1,
                ),

                // Increase memory cell value
                b'+' => Interpreter::process_workspace_inc(
                    &mut workspace,
                    &mut ops,
                    1,
                ),

                // Decrease memory cell value
                b'-' => Interpreter::process_workspace_inc(
                    &mut workspace,
                    &mut ops,
                    -1,
                ),

                // Output the value of the current memory cell
                b'.' => {
                    // Commit and add a new operator
                    Interpreter::commit(&mut workspace, &mut ops, None);
                    ops.push(Op::Output);
                },

                // Read user input
                b',' => {
                    // Commit and add a new operator
                    Interpreter::commit(&mut workspace, &mut ops, None);
                    ops.push(Op::Input);
                },

                // Start a conditional loop
                b'[' => {
                    // Commit and add a new conditional routine
                    Interpreter::commit(&mut workspace, &mut ops, None);
                    ops.push(
                        Interpreter::interpret_routine(bytes, true),
                    );
                },

                // End a conditional loop, finish this operation vector
                b']' => break,

                // Unrecognized operation, skip
                _ => continue,
            }
        }

        // Commit the last workspace operation
        if let Some(op) = workspace {
            ops.push(op);
        }

        ops
    }

    /// Commit the given workspace in the given.
    /// And reinitialize the workspace with the given `fresh` operator.
    /// This is quicker than first setting it to zero.
    ///
    /// This method is intended to be used internally.
    ///
    /// The `workspace` is committed to `ops` if set.
    /// This leaves `workspace` with `fresh`.
    ///
    /// You may want to consider using `None` as `fresh` option,
    /// to reset the workspace.
    fn commit(workspace: &mut Option<Op>, ops: &mut Vec<Op>, fresh: Option<Op>) {
        // Take the workspace item, put it in the list
        if let Some(op) = mem::replace(workspace, fresh) {
            ops.push(op);
        }
    }

    /// Process a seek instruction, in the context of the given workspace.
    ///
    /// The workspace may be used to combine this new instruction with,
    /// as optimization.
    ///
    /// If an incompatible instruction was in the workspace, the workspace is
    /// committed, and a new workspace is created with the preferred
    /// instruction.
    ///
    /// If the workspace was compatible, the workspace will be left uncommitted
    /// for possible further optimizations in upcomming instructions.
    ///
    /// The `workspace` is committed to `ops`.
    fn process_workspace_seek(
        workspace: &mut Option<Op>,
        ops: &mut Vec<Op>,
        amount: isize,
    ) {
        // Determine whether to combine to an existing workspace,
        // or to commit and define a new operator workspace
        match *workspace {
            // Combine with the workspace operation
            Some(
                Op::Seek {
                    amount: ref mut current,
                }
            ) => *current += amount,

            // Commit the workspace, start working on a new seek operator
            _ => Interpreter::commit(
                workspace,
                ops,
                Some(
                    Op::Seek { amount }
                ),
            ),
        }
    }

    /// Process a increment instruction, in the context of the given workspace.
    ///
    /// The workspace may be used to combine this new instruction with,
    /// as optimization.
    ///
    /// If an incompatible instruction was in the workspace, the workspace is
    /// committed, and a new workspace is created with the preferred
    /// instruction.
    ///
    /// If the workspace was compatible, the workspace will be left uncommitted
    /// for possible further optimizations in upcomming instructions.
    ///
    /// The `workspace` is committed to `ops`.
    fn process_workspace_inc(
        workspace: &mut Option<Op>,
        ops: &mut Vec<Op>,
        amount: isize,
    ) {
        // Determine whether to combine to an existing workspace,
        // or to commit and define a new operator workspace
        match *workspace {
            // Combine with the workspace operation
            Some(
                Op::Inc {
                    amount: ref mut current,
                }
            ) => *current += amount,

            // Commit the workspace, start working on a new increment operator
            _ => Interpreter::commit(
                workspace,
                ops,
                Some(
                    Op::Inc { amount }
                ),
            ),
        }
    }
}



/// The memory bank of a brainfuck program.
///
/// This struct defines the state of such a program,
/// and provides helper functions to easily manage it.
struct Memory {
    /// The memory data set
    data: [u8; MEM_SIZE],

    /// Index of the current memory cell pointer
    pointer: usize,
}

impl Memory {
    /// Create new application memory.
    ///
    /// This allocates all memory the program might use,
    /// and returns the initial memory state.
    pub fn new() -> Memory {
        Memory {
            data: [0; MEM_SIZE],
            pointer: 0,
        }
    }

    /// Seek the memory cell pointer for the given relative `amount`.
    pub fn seek(&mut self, amount: isize) {
        // // TODO: Is this correct
        // // Seek if the value won't underflow
        // if amount >= 0 || self.pointer as isize >= -amount {
        //     self.pointer += amount as usize;
        // } else {
        //     self.pointer = 0;
        // }
        self.pointer += amount as usize;
    }

    /// Increase the value of the current memory cell by the given relative
    /// `amount`.
    pub fn inc(&mut self, amount: isize) {
        // TODO: Don't go out of bound
        // TODO: Don't cast
        self.data[self.pointer] += amount as u8;
    }

    /// Read and return the value of the current memory cell.
    pub fn read(&self) -> u8 {
        self.data[self.pointer]
    }

    /// Check whether the current memory cell is zero.
    pub fn zero(&self) -> bool {
        self.data[self.pointer] == 0
    }

    /// Set the current memory cell value to zero.
    pub fn set_zero(&mut self) {
        self.data[self.pointer] = 0;
    }
}



/// Operation types, supported by this interpreter.
/// This may be considered an intermediate operation set.
///
/// There are many more (and different) types of operations than the brainfuck
/// specification supports. This allows fine grained optimization at
/// interpretation time.
///
/// Brainfuck programs are translated into these operations,
/// which will define the program structure in-memory for quick execution.
enum Op {
    /// A routine wrapping other operations.
    /// This routine may be simple, or it may be conditional with makes the
    /// routine loopable.
    Routine {
        /// A set of operations contained by this routine
        ops: Vec<Op>,

        /// Defines whether this routine is loopable.
        ///
        /// `true` if this routine is contitionally loopable.
        /// `false` if it isn't.
        cond: bool,
    },

    /// Seek the memory pointer for the relative `amount`.
    Seek {
        /// Seek amount
        amount: isize
    },

    /// Increment the value in the current memory cell with the relative
    /// `amount`.
    Inc {
        /// Increase amount
        amount: isize
    },

    /// Put a byte from user input into the current memory cell.
    Input,

    /// Output the value of the current memory cell.
    Output,

    /// Set the value of the current memory cell to zero.
    Zero,
}

impl Op {
    /// Execute the current operation.
    ///
    /// If this operation is a conditional routine, the condition is properly
    /// evaluated as expected.
    ///
    /// The given `memory` and `output` objects are used to execute these
    /// operations on, if relevant.
    pub fn execute(&self, memory: &mut Memory, output: &mut Vec<u8>) {
        // Invoke operation specific logic
        match *self {
            // Seek the memory cell pointer
            Op::Seek { amount } => memory.seek(amount),

            // Increase the value in the current memory cell
            Op::Inc { amount } => memory.inc(amount),

            // Invoke a routine
            Op::Routine {
                ref ops,
                cond,
            } => {
                // If conditional, skip the routine if the current memory cell
                // value is zero
                if cond && memory.zero() {
                    return;
                }

                // Keep looping the routine until the end condition is reached
                loop {
                    // Execute all contained operations
                    ops.iter().for_each(|op| op.execute(memory, output));

                    // End if not conditional, or if the current memory cell
                    // value is zero
                    if !cond || memory.zero() {
                        break;
                    }
                }
            },

            // Set the value of the current memory cell to zero
            Op::Zero => memory.set_zero(),

            // Output the value of the current memory cell
            Op::Output => output.push(memory.read()),

            // Handle user input
            Op::Input => panic!("Input not yet supported"),
        }
    }
}
