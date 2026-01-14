use std::{error::Error, path::PathBuf};

/// Arguments parsed from the command line.
pub struct Args<'a> {
    /// Path to the root filesystem.
    pub rootfs: Option<PathBuf>,
    /// Docker image name to pull/use.
    pub image: Option<&'a str>,
    /// Volume path on the host to mount into the container.
    pub volume: Option<PathBuf>,
    /// Command to execute inside the container, along with its arguments.
    pub command: &'a [String],
}

const USAGE: &str = "MyMoulette, the students'nightmare, now highly secured
 Usage: ./mymoulette [-v student_workdir] <-I docker-img|rootfs-path>
 moulette_prog [moulette_arg [...] ]
    rootfs-path is the path to the directory containing the new rootfs (exclusive
 with -I option)
    docker-img is an image available on hub.docker.com (exclusive with rootfs path)
 moulette_prog will be the first program to be launched, must already be in
 the environment
    student_workdir is the directory containing the code to grade";

/// Parses command line arguments.
///
/// Handles custom flags for volume (-v), image (-I), and rootfs (-R).
/// The remaining arguments are treated as the command to execute.
///
/// # Arguments
///
/// * `args` - Command line arguments.
///
/// # Returns
///
/// Returns `Ok(Some(Args))` if parsing is successful.
/// Returns `Ok(None)` if help flag (-h) is present.
/// Returns `Err` if required arguments are missing or invalid.
pub fn parse_args<'a>(args: &'a [String]) -> Result<Option<Args<'a>>, Box<dyn Error>> {
    let mut rootfs = None;
    let mut image = None;
    let mut volume = None;

    let mut i = 0;
    let len = args.len();

    while i < len {
        let arg = args[i].as_str();

        match arg {
            "-h" => {
                println!("{}", USAGE);
                return Ok(None);
            }
            "-v" => {
                i += 1;
                if i >= len {
                    return Err("Missing value for -v".into());
                }
                volume = Some(PathBuf::from(&args[i]));
            }
            "-I" => {
                i += 1;
                if i >= len {
                    return Err("Missing value for -I".into());
                }
                image = Some(args[i].as_str());
            }
            "-R" => {
                i += 1;
                if i >= len {
                    return Err("Missing value for -R".into());
                }
                rootfs = Some(PathBuf::from(&args[i]));
            }
            _ => {
                let command_slice = &args[i..];

                if rootfs.is_none() && image.is_none() {
                    return Err("You must provide a rootfs path or use -I <image>".into());
                }

                return Ok(Some(Args {
                    rootfs,
                    image,
                    volume,
                    command: command_slice,
                }));
            }
        }
        i += 1;
    }

    Err("You must provide a command to execute".into())
}
