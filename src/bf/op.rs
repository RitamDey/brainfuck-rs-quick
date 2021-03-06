use super::tty_read::TermReader;

use super::Memory;
use super::Options;



/// Operation types, supported by this interpreter.
/// This may be considered an intermediate operation set.
///
/// There are many more (and different) types of operations than the brainfuck
/// specification supports. This allows fine grained optimization at
/// interpretation time.
///
/// Brainfuck programs are translated into these operations,
/// which will define the program structure in-memory for quick execution.
#[derive(Debug)]
pub enum Op {
    /// A routine wrapping other operations.
    /// This routine may be simple, or it may be conditional with makes the
    /// routine loopable.
    ///
    /// The fist property defines the set of operations countained by this
    /// routine.
    /// The second property defines whether this routine is
    /// conditional/loopable.
    /// - `true` if this routine is contitionally loopable.
    /// - `false` if it isn't.
    Routine(Vec<Op>, bool),

    /// Seek the memory pointer for the relative amount.
    Seek(isize),

    /// Increment the value in the current memory cell with the relative
    /// amount.
    Inc(isize),

    /// Put a byte from user input into the current memory cell.
    Input,

    /// Output the value of the current memory cell.
    Output,

    /// Set the value of the current memory cell to zero.
    Zero,

    /// Add the current cell value to the given relative targets,
    /// zeroing the current cell.
    AddAndZero(Vec<(isize, f32)>),
}

impl Op {
    /// Execute the current operation.
    ///
    /// If this operation is a conditional routine, the condition is properly
    /// evaluated as expected.
    ///
    /// The given `memory` and `output` objects are used to execute these
    /// operations on, if relevant.
    pub fn execute(&self, memory: &mut Memory, options: &Options, output: &mut Vec<u8>) {
        // Invoke operation specific logic
        match *self {
            // Seek the memory cell pointer
            Op::Seek(amount) => memory.seek(amount),

            // Increase the value in the current memory cell
            Op::Inc(amount) => memory.inc(amount),

            // Invoke a routine
            Op::Routine(ref ops, cond) => {
                // If conditional, skip the routine if the current memory cell
                // value is zero
                if cond && memory.zero() {
                    return;
                }

                // Keep looping the routine until the end condition is reached
                loop {
                    // Execute all contained operations
                    ops.iter().for_each(|op| op.execute(memory, options, output));

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
            Op::Output => {
                // Read the value, and push it to the output
                let value = memory.read();
                output.push(value);

                // If not buffered, print the value immediately
                if !options.buffer {
                    print!("{}", value as char);
                }
            },

            // Handle user input
            Op::Input => memory.write(
                TermReader::open_stdin(&options.reader_options)
                    .expect("failed to open user input reader")
                    .read_byte()
                    .expect("failed to read user input")
            ),

            // Add the current cell value to others, and zero
            Op::AddAndZero(ref targets) => memory.copy_zero(targets),
        }
    }
}
