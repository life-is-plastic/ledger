mod cmds;
mod output;
mod sharedopts;
mod util;

pub use cmds::main;
use output::Output;

#[cfg(test)]
mod testing;
