use clap::Parser;
use gather::gather_characters;
use hdf5::File;
use std::fs::File as StdFile;
mod gather;

#[derive(Parser, Debug)]
#[command(name = "Character Gather")]
#[command(version = "0.0.1")]
#[command(about = "Takes in text files and analyses how often character appear after each other", long_about = None)]
struct Args {
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    acceptable_types: Vec<char>,
    #[arg(long, default_value_t = 4)]
    offset_back: isize,
    #[arg(long, default_value_t = 4)]
    offset_front: isize,
    #[arg(short)]
    input: String,
    #[arg(short)]
    output: String,
}

fn main() -> hdf5::Result<()> {
    let args = Args::parse();
    let file = StdFile::open(args.input).expect("Could not open input file");

    println!("Accepted types are:\n{:?}", args.acceptable_types);

    let hdf5_file = File::create(args.output).expect("Could not create file");
    let dataset = hdf5_file
        .new_dataset::<u64>()
        .shape((
            args.acceptable_types.len(),
            args.acceptable_types.len(),
            args.offset_back as usize + args.offset_front as usize + 1,
        ))
        .create("results")
        .expect("could not create base");

    let data = gather_characters(
        args.acceptable_types,
        args.offset_back,
        args.offset_front,
        file,
    );

    dataset.write(&data).expect("Could not write the fiel");
    Ok(())
}
