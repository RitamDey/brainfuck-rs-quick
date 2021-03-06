//! # Add and Zero optimization
//! This optimization optimizes routines that add their current value to some
//! other number of cells, and then zero their own cell.
//! This routine is quite common, and has great optimizability as the whole
//! routine can be removed, transforming it into a single operation.
//!
//! This generic implementation supports many forms of this routine.
//! - Any number of other cells can be added to or subtracted from.
//! - Other cells can be at any position.
//! - To various cells a value may be added or subtracted with a different
//!   factor.
//!
//! This optimization therefore covers many forms of copy and multiplication
//! routines.
//!
//! # Optimization requirements
//! - There must be at least 4 operations.
//! - Subtraction must be the first operation.
//! - The following operations must be any number of the following sequence:
//!     - Seek to another cell to add to or subtract from
//!     - Add or subtract
//! - The last operation must seek back to the base cell.
//! - The base cell must never be touched by an other operation.
//!
//! # Example routines
//! - `[->+<]'
//!     - Add the current cell value to the next cell
//!     - Zero the current cell
//! - `[->>>+<<<]`
//!     - Add the current cell value to the 3rd next cell
//!     - Zero the current cell
//! - `[->+>+>+<<<]`
//!     - Add the current cell value to the next 3 cells
//!     - Zero the current cell
//! - `[-<<+>+>>>+<+<]`
//!     - Add the current cell value to cell: -2, -1, 1, 2
//!     - Zero the current cell
//! - `[--->+++>+++<<]`
//!     - Add the current cell value to the next two, in steps of 3
//!     - Zero the current cell
//! - `[->+>++>-<<<]`
//!     - Add the current value to the next cell
//!     - Add the current value to the 2nd next cell times 2
//!     - Zero the current and 3rd cell
//! - `[--->+<<+++>]`
//!     - Add a 1/3e of the current cell to the next cell
//!     - Add the current cell value to the previous cell
//!     - Zero the current cell



use super::super::super::Op;



/// Optimize addition and zero routines.
///
/// This optimization is applied on routines.
/// True or false should be given to `cond` depending on whether the routine
/// is conditional or not.
/// The operations contained by the routine should be given to `ops`.
///
/// If `Some` is returned, the whole routine should be replaced by it's
/// contents.
pub fn optimize_add_and_zero(cond: bool, ops: &Vec<Op>) -> Option<Op> {
    // Do not run if this isn't a conditional loop,
    // there must be at least six operations
    if !cond || ops.len() < 4 {
        return None;
    }

    // Create a operations iterator
    let mut iter = ops.iter().enumerate();

    // The first cell must subtract
    let step;
    match iter.next() {
        Some((_, &Op::Inc(amount))) if amount < 0 => step = -amount,
        _ => return None,
    }

    // Create a vector for relative memory positions to copy to,
    // and a variable that remembers the current offset
    let mut targets = Vec::with_capacity((ops.len() - 2) / 2);
    let mut offset = 0;

    // Find cells this data is moved into
    loop {
        // Get the seek and subtract operators
        let (_, seek_op) = iter.next()?;
        let (sub_i, sub_op) = iter.next()?;

        // This must be a seek, modify the offste
        match seek_op {
            &Op::Seek(amount) => offset += amount,
            _ => return None,
        }

        // This must add/subtract, remember the factor to do it with
        let factor;
        match sub_op {
            &Op::Inc(amount) =>
                factor = if amount != 0 {
                    amount as f32 / step as f32
                } else {
                    0f32
                },
            _ => return None,
        }

        // We may not overwrite the base cell
        if offset == 0 {
            return None;
        }

        // Add the current offset to the target list
        if factor != 0f32 {
            targets.push((offset, factor));
        }

        // If the subtraction was the second last operator,
        // the last must set the offset back to zero
        if sub_i == ops.len() - 2 {
            // Get the reset operator
            let (_, reset_op) = iter.next()?;

            // This must be addition
            match reset_op {
                &Op::Seek(amount) if amount == -offset=> {},
                _ => return None,
            }

            // This optimization is succesful, return the resulting operator
            return Some(
                Op::AddAndZero(targets),
            );
        }
    }
}
