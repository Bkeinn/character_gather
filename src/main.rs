use clap::{Parser, Subcommand};
use gather::gather_characters;
use hdf5::{
    self,
    types::{VarLenArray, VarLenAscii},
};
use ndarray::{Array2, Array3, ArrayBase, Dim, OwnedRepr};
use normalize::normalizer_sum_one;
use std::fs::File as StdFile;
mod char_dataset;
mod gather;
mod normalize;
mod threading;

use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[command(name = "Character Gather")]
#[command(version = "0.0.2")]
#[command(about = "Takes in text files and analyses how often character appear after each other", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    // #[arg(long)]
    // normalize: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    AbsoluteCharRelation {
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
        #[arg(short)]
        threads: usize,
    },
    Normalize {
        #[arg(short)]
        input: String,
        #[arg(
            long,
            default_value_t = 0,
            help = "Set the normalization type:\n\t0: Value = (X - min)/(max - min)\n\t1: Value = X/sum\n\t:2 Value = X - mean\n\t3: Value = X / max(X)\n\t4: Value = (X - mean) / std_deviation"
        )]
        n_type: u8,
    },
    CharDataset {
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
        #[arg(short)]
        threads: usize,
    },
}

fn main() -> hdf5::Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::AbsoluteCharRelation {
            acceptable_types,
            offset_back,
            offset_front,
            input,
            output,
            threads,
        }) => {
            let file = StdFile::open(input).expect("Could not open input file");

            let hdf5_file = hdf5::File::create(output).expect("Could not create file");
            let dataset = hdf5_file
                .new_dataset::<u64>()
                .shape((
                    acceptable_types.len(),
                    acceptable_types.len(),
                    offset_back as usize + offset_front as usize + 1,
                ))
                .create("absolute_data")
                .expect("could not create base");

            dataset
                .new_attr::<VarLenAscii>()
                .shape(())
                .create("acceptable_types")?
                .write_scalar(
                    &VarLenAscii::from_ascii(&acceptable_types.iter().collect::<String>()).unwrap(),
                )
                .expect("Could not create acceptable types attribute");
            let data =
                gather_characters(acceptable_types, offset_back, offset_front, file, threads);
            dataset.write(&data).expect("Could not write the fiel");
            dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_back")?
                .write_scalar(&offset_back)
                .expect("Could not create offset_back attribute");
            dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_front")?
                .write_scalar(&offset_front)
                .expect("Could not create offset_front attribute");
        }
        Some(Commands::Normalize { input, n_type }) => {
            let hdf5_file = hdf5::File::open_as(input, hdf5::file::OpenMode::ReadWrite)
                .expect("Could not find file {input}");
            let absolute_dataset = match hdf5_file.dataset("/absolute_data/") {
                Ok(dataset) => dataset,
                Err(e) => match hdf5_file.dataset("/results/") {
                    Ok(dataset) => dataset,
                    Err(oe) => panic!("Could not find the dataset in this file: {e} | {oe}"),
                },
            };
            let offset_front: u64 = absolute_dataset.attr("offset_front")?.read_scalar()?;
            let offset_back: u64 = absolute_dataset.attr("offset_back")?.read_scalar()?;
            let acceptable_types: VarLenAscii =
                absolute_dataset.attr("acceptable_types")?.read_scalar()?;
            let acceptable_types = acceptable_types
                .as_bytes()
                .iter()
                .map(|c| *c as char)
                .collect::<Vec<char>>();

            let data: Array3<u64> = absolute_dataset.read()?;
            let normalized_data = match n_type {
                0 => normalize::normalizer_min_max(data),
                1 => normalize::normalizer_sum_one(data),
                2 => normalize::normalizer_minus_mean(data),
                3 => normalize::normalizer_divide_max(data),
                4 => normalize::normalizer_z_score(data),
                _ => panic!("No such normalizer implemented"),
            };

            let normalized_dataset = match hdf5_file
                .new_dataset::<f64>()
                .shape((
                    acceptable_types.len(),
                    acceptable_types.len(),
                    offset_back as usize + offset_front as usize + 1,
                ))
                .create("normalized_data")
            {
                Ok(dataset) => dataset,
                Err(_) => hdf5_file
                    .dataset("/normalized_data/")
                    .expect("Could not open normalized_data"),
            };
            normalized_dataset
                .write(&normalized_data)
                .expect("Could not write data to dataset");

            let attr = match normalized_dataset
                .new_attr::<VarLenAscii>()
                .shape(())
                .create("acceptable_types")
            {
                Ok(attr) => attr,
                Err(_) => normalized_dataset.attr("acceptable_types")?,
            };
            attr.write_scalar(
                &VarLenAscii::from_ascii(&acceptable_types.iter().collect::<String>()).unwrap(),
            )?;

            let attr = match normalized_dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_back")
            {
                Ok(attr) => attr,
                Err(_) => normalized_dataset.attr("offset_back")?,
            };
            attr.write_scalar(&offset_back)
                .expect("Could not create offset_back attribute");
            let attr = match normalized_dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_front")
            {
                Ok(attr) => attr,
                Err(_) => normalized_dataset.attr("offset_front")?,
            };
            attr.write_scalar(&offset_front)
                .expect("Could not create offset_front attribute");
        }
        Some(Commands::CharDataset {
            acceptable_types,
            offset_back,
            offset_front,
            input,
            output,
            threads,
        }) => {
            let file = StdFile::open(input).expect("Could not open input file");

            let hdf5_file = hdf5::File::create(output).expect("Could not create file");

            for character in acceptable_types.clone() {
                let name = format!("CharDataset_{}.csv", &character);
                let result_file = StdFile::create(&name).expect("Could not create File");
                let mut result_file = OpenOptions::new().append(true).open(name).unwrap();

                char_dataset::gather_dataset(
                    character,
                    acceptable_types.clone(),
                    offset_back,
                    offset_front,
                    file.try_clone().unwrap(),
                    threads,
                    result_file,
                );

                // let data: Vec<Vec<u8>> = char_dataset::gather_dataset(
                //     character,
                //     acceptable_types.clone(),
                //     offset_back,
                //     offset_front,
                //     file.try_clone().unwrap(),
                //     threads,
                // )
                // .into_iter()
                // .map(|vec| vec.into_iter().map(|c| c as u8).collect::<Vec<u8>>())
                // .collect();
                // let dims = (data.len(), data[0].len());
                // let data: Array2<u8> =
                //     Array2::from_shape_vec(dims, data.into_iter().flatten().collect()).unwrap();

                // let dataset = hdf5_file
                //     .new_dataset::<u8>()
                //     .shape(dims)
                //     .create(&*name)
                //     .unwrap();
                // dataset.write(&data).unwrap();
            }

            let dataset = hdf5_file
                .new_dataset::<u64>()
                .shape((
                    acceptable_types.len(),
                    acceptable_types.len(),
                    offset_back as usize + offset_front as usize + 1,
                ))
                .create("absolute_data")
                .expect("could not create base");

            dataset
                .new_attr::<VarLenAscii>()
                .shape(())
                .create("acceptable_types")?
                .write_scalar(
                    &VarLenAscii::from_ascii(&acceptable_types.iter().collect::<String>()).unwrap(),
                )
                .expect("Could not create acceptable types attribute");
            let data =
                gather_characters(acceptable_types, offset_back, offset_front, file, threads);
            dataset.write(&data).expect("Could not write the fiel");
            dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_back")?
                .write_scalar(&offset_back)
                .expect("Could not create offset_back attribute");
            dataset
                .new_attr::<u64>()
                .shape(())
                .create("offset_front")?
                .write_scalar(&offset_front)
                .expect("Could not create offset_front attribute");
        }
        None => eprint!("No command given, so nothing will happen"),
    }

    Ok(())
}
