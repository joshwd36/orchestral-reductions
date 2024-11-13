mod fraction;
mod music_xml;
mod output_score;
mod phrase;
mod phrase_element;
mod score_representation;

use clap::{App, Arg};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::process;

fn main() {
    // Specify the available command line arguments.
    let app = App::new("Orchestral Reductions")
        .arg(
            Arg::with_name("input")
                .short("i")
                .help("The input MusicXML file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .help("The output MusicXML file")
                .takes_value(true)
                .default_value("output.musicxml"),
        )
        .arg(
            Arg::with_name("merge-by-average")
                .short("a")
                .help("Use the Merge By Average transformation instead of Distribute Staves"),
        )
        .arg(
            Arg::with_name("no-merge")
                .short("m")
                .help("Don't merge phrases together in the output")
                .conflicts_with("merge-by-average"),
        )
        .arg(
            Arg::with_name("no-adjust-octaves")
                .short("n")
                .help("Don't adjust octaves to ensure the piece fits within a handspan"),
        )
        .arg(
            Arg::with_name("handspan")
                .short("h")
                .help("The maximum stretch permissible within a stave in semitones")
                .conflicts_with("no-adjust-octaves")
                .takes_value(true)
                .default_value("12"),
        )
        .arg(
            Arg::with_name("staves")
                .short("s")
                .help("The number of staves to use in the output.")
                .takes_value(true)
                .default_value("2"),
        )
        .arg(
            Arg::with_name("max-phrase-length")
                .short("l")
                .help("The maximum length of a phrase, in bars. Use 0 to indicate no maximum")
                .takes_value(true)
                .default_value("1"),
        );

    // Parse the command line arguments.
    let matches = app.get_matches();
    let input_filename = matches.value_of("input").unwrap();
    let output_filename = matches.value_of("output").unwrap();
    let merge_by_average = matches.is_present("merge-by-average");
    let no_merge = matches.is_present("no-merge");
    let no_adjust_octaves = matches.is_present("no-adjust-octaves");

    let staves: u8 = matches
        .value_of("staves")
        .unwrap()
        .parse()
        .unwrap_or_else(|_err| {
            println!("Number of staves must be an integer");
            process::exit(1)
        });
    let phrase_len: u32 = matches
        .value_of("max-phrase-length")
        .unwrap()
        .parse()
        .unwrap_or_else(|_err| {
            println!("Maximum phrase length must be an integer");
            process::exit(1)
        });

    let mut input_file = fs::File::open(input_filename).unwrap_or_else(|err| {
        println!(
            "Could not open file {}, failed with error: {}",
            input_filename, err
        );
        process::exit(1)
    });

    let mut text = String::new();
    input_file.read_to_string(&mut text).unwrap_or_else(|err| {
        println!(
            "Could not open file {}, failed with error: {}",
            input_filename, err
        );
        process::exit(1)
    });

    let doc = roxmltree::Document::parse(&text).unwrap_or_else(|err| {
        println!(
            "Could not parse file {}, failed with error: {}",
            input_filename, err
        );
        process::exit(1)
    });

    let mut parser = score_representation::ScoreParser::new(doc);
    let phrase_list = parser.parse_score(phrase_len);
    let stave_list = if merge_by_average {
        phrase_list.merge_by_average(staves)
    } else {
        let mut stave_list = phrase_list.distribute_staves(staves);
        if !no_adjust_octaves {
            let handspan: u32 = matches
                .value_of("handspan")
                .unwrap()
                .parse()
                .unwrap_or_else(|_err| {
                    println!("Handspan must be an integer");
                    process::exit(1)
                });
            if handspan < 12 {
                println!("Handspan must be greater than or equal to 12 semitones");
                process::exit(1)
            }
            stave_list.adjust_octaves(handspan);
        }
        stave_list
    };

    let stave_list = if no_merge {
        stave_list
    } else {
        stave_list.merge()
    };
    let output = output_score::OutputScore::new(stave_list);
    let mut output_file = fs::File::create(output_filename).unwrap_or_else(|err| {
        println!(
            "Could not create output file {}, failed with error: {}",
            output_filename, err
        );
        process::exit(1)
    });
    output_file
        .write_all(&output.get_value())
        .unwrap_or_else(|err| {
            println!(
                "Could not write to output file {}, failed with error: {}",
                output_filename, err
            );
            process::exit(1)
        });
}
