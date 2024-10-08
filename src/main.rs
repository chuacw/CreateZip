/*
    Original code by Chee-Wee, Chua
    9 Aug 2024,
    Singapore
 */

use std::{env};
use std::fs::{File, remove_file, rename};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, ZipWriter, CompressionMethod, result::ZipError, read::ZipArchive};
use zip::write::{ExtendedFileOptions};
use std::collections::HashSet;
use tempfile::{NamedTempFile};

fn call_three_functions<F1, F2, F3>(func1: F1, mut func2: F2, func3: F3)
where F1: Fn(), F2: FnMut(), F3: Fn()
{
    func1();
    func2();
    func3();
}

/// Get temporary filename in a given directory
fn get_temp_file_name(directory: String) -> String {
    let temp_zip_file = NamedTempFile::new_in(directory).unwrap();
    let temp_zip_file_name = temp_zip_file.path().to_str().unwrap().to_string();
    temp_zip_file_name
}

fn add_or_replace_files(zip_file_name: &str, files_to_add: &[PathBuf]) -> zip::result::ZipResult<()> {
    // Create a temporary file to store the modified ZIP contents
    let mut temp_zip_file_name = get_temp_file_name(Path::new(zip_file_name).parent().unwrap().to_str().unwrap().to_string());
    let temp_zip_file = File::create(temp_zip_file_name.clone()).unwrap();
    let mut zip_writer = ZipWriter::new(temp_zip_file);

    // Open the existing ZIP file
    let zip_file = File::open(zip_file_name).unwrap();
    let mut zip_archive = ZipArchive::new(zip_file)?;

    // Collect existing files and their order
    let mut existing_files_within_zip = Vec::new();
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i)?;
        existing_files_within_zip.push(file.name().to_string());
    }

    // Track files to replace
    let files_to_replace: HashSet<String> = files_to_add
        .iter()
        .filter_map(|p| p.file_name().map(|name| name.to_string_lossy().to_string()))
        .collect();

    // Copy all files from the existing ZIP archive to the new ZIP archive
    // in the original order, but replace files as needed
    for file_name in existing_files_within_zip.clone() {
        if files_to_replace.contains(&file_name) {
            // Replace file
            let file_to_add = files_to_add
                .iter()
                .find(|p| p.file_name().unwrap().to_string_lossy() == file_name)
                .unwrap();
            let file_path = Path::new(file_to_add);
            let zip_option: FileOptions<ExtendedFileOptions> = FileOptions::default().compression_method(CompressionMethod::Deflated);
            print!("Replacing {}", file_name.clone().to_string());
            zip_writer.start_file(file_name.clone().to_string(), zip_option)?;
            let mut file = File::open(file_path)?;
            std::io::copy(&mut file, &mut zip_writer)?;
            println!("... replaced!");
        } else {
            // Copy existing file
            call_three_functions(
                || {
                    print!("Copying {}", file_name.clone().to_string());
                },
                || {
                    let zip_option: FileOptions<ExtendedFileOptions> = FileOptions::default().compression_method(CompressionMethod::Deflated);
                    zip_writer.start_file(file_name.clone(), zip_option).expect("Failed!");
                    let mut file = zip_archive.by_name(&file_name).expect("Failed!");
                    std::io::copy(&mut file, &mut zip_writer).expect("Failed!");
                },
                || {
                    println!("... copied!");
                }
            );
        }
    }

    // Add any new files that are not already in the zip
    for file_path in files_to_add {
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        if !existing_files_within_zip.contains(&file_name) {
            call_three_functions(
                || {
                    print!("Adding {}", file_name.clone().to_string());
                },
                || {
                    let zip_option: FileOptions<ExtendedFileOptions> = FileOptions::default().compression_method(CompressionMethod::Deflated);
                    zip_writer.start_file(file_name.clone().to_string(), zip_option).expect("Failed!");
                    let mut file_to_add = File::open(file_path).expect("Failed!");
                    std::io::copy(&mut file_to_add, &mut zip_writer).expect("Failed!");
                },
                ||  {
                    println!("... added!");
                }
            )
        }
    }

    // Finish the new ZIP file
    let _ = zip_writer.finish();

    // Replace the original ZIP file with the modified one
    // drop(zip_writer); // Ensure all data is written to the file

    remove_file(zip_file_name)?; // Remove the original ZIP file

    let rename_res = rename(temp_zip_file_name.clone(), zip_file_name); // Rename temp file to original name
    match rename_res {
        Ok(_) => {
            // do nothing
        },
        Err(e) => {
            println!("{:?}", e);
            let x= e.raw_os_error().unwrap();
            println!("{:?}", x);
            // assume it's a CrossesDevices error ->The system cannot move the file to a different disk drive.
            let copy_res = std::fs::copy(temp_zip_file_name.clone(), zip_file_name);
            if let Err(e) = copy_res {
                // Some failure
                println!("Failure occurred: {:?}", e);
            } else {
                std::fs::remove_file(temp_zip_file_name)?;
            }
        }
    }
    Ok(())
}

fn create_zip_file() -> Result<(), ZipError> {
    let args: Vec<String> = env::args().collect();
    let arg_len = args.len();
    match arg_len {
        0..=1 => {
            println!("Nothing to do!");
            return Ok(());
        },
        _ => {
            let zip_filename = &args[1];
            let zip_filename_path = Path::new(zip_filename);
            match arg_len {
                2 => {
                    if zip_filename_path.exists() {
                        println!("Listing contents of {}", zip_filename);
                        println!("{}", "-".repeat(40));

                        let file = File::open(zip_filename)?;
                        let mut archive = ZipArchive::new(file)?;

                        for i in 0..archive.len() {
                            let file = archive.by_index(i)?;
                            println!("{}", file.name());
                        }
                    }

                    return Ok(());
                },
                _ => {
                    let open_mode = if zip_filename_path.exists() {
                        "Updating"
                    } else {
                        "Creating"
                    };

                    let msg = format!("{} {}", open_mode, zip_filename);
                    println!("{}", msg);
                    println!("{}", "-".repeat(msg.len()));

                    let filepath = Path::new(zip_filename_path);
                    match filepath.exists() {
                        false => {
                            let file = File::create(&zip_filename_path)?;
                            let mut zip = ZipWriter::new(file);
                            let mut files_to_add = Vec::new();
                            for filename in &args[2..] {
                                files_to_add.push(PathBuf::from(filename));
                                if Path::new(filename).exists() {
                                    print!("Adding {}", filename);
                                    let mut buffer = Vec::new();
                                    let mut f = File::open(filename)?;
                                    f.read_to_end(&mut buffer)?;
                                    let filename = Path::new(filename).file_name().unwrap().to_str().unwrap();
                                    let zip_option: FileOptions<ExtendedFileOptions> = FileOptions::default().compression_method(CompressionMethod::Deflated);
                                    zip.start_file(filename, zip_option).expect("TODO: panic message");
                                    zip.write_all(&buffer)?;
                                    println!("... added!");
                                } else {
                                    println!("Unable to find {}", filename);
                                }
                            }
                            zip.finish()?;
                        },
                        true => {
                            let mut files_to_add = Vec::new();
                            for filename in &args[2..] {
                                files_to_add.push(PathBuf::from(filename));
                            }
                            let _ = add_or_replace_files(zip_filename, &files_to_add);
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

fn main() {
    if let Err(e) = create_zip_file() {
        eprintln!("Error: {}", e);
    }
}
