//use core::time;
use std::path::PathBuf;
//use std::fmt::format;

// use std::error::Error;
// use std::fmt::Display;
use clap::Parser;
#[macro_use(c)]
extern crate cute;
use colored::Colorize;
use std::fmt;
use rayon::prelude::*;

#[derive(Parser)]
#[command(about = format!("{}","RAW timestamper".bold().green().underline()), 
    long_about = format!("{}: adds a timestamp to the beginning of RAW (and their XMP sidecar) files",
        "RAW timestamper".bold().green().underline())
)]
        struct Cli {
            #[arg(help = "List of RAW files to rename. Any XMP sidecars present will \
                be renamed alongside their corresponding RAWs")]
            files: Vec<PathBuf>,
            //#[arg(short,long)]
            //verbose: bool,
            #[arg(short = 'n', long = "dry-run", 
                help = "Perform a trial run with no changes made")]
            dryrun: bool,
            #[arg(short = 'd',long = "datestamp", 
                help = "Use datestamps (YYYY-MM-DD) without the time of day information")]
            datestamp: bool,
            #[arg(short = 'q',long = "quiet")]
            quiet:bool,
            #[arg(short = 'b', long = "benchmark")]
            benchmark:bool,
            // #[arg(short = 'p',long = "proceed", 
            //     help = "proceed ignoring files with errors")]
            // proceed: bool,
            // #[arg(short = 'o', long = "overwrite")]
            // overwrite:bool,
        }

struct ReplaceAction{
    o_raw: PathBuf,
    //o_raw_p: bool,
    n_raw_p: bool,
    timestamp: Option<String>,
    o_xmp_p: bool,
    n_xmp_p: bool,
    time: std::time::Duration
/* valid states:
.                    ORP  NRP  TS  OXP  NXP
old_raw not present  F    F    N   F    F
old_raw not valid    T    F    N   F    F
ow RAW, no XMP       T    T    S   F    F
ow RAW, XMP          T    T    S   T    F
ow RAW, ow XMP       T    T    S   T    T
RAW, no XMP          T    F    S   F    F   <--- happy path
RAW, XMP             T    F    S   T    F   <--- happy path
RAW, ow XMP          T    F    S   T    T  */
}

impl ReplaceAction{
    fn o_raw_p(&self) -> bool {self.o_raw.exists() && self.o_raw.is_file()}
    fn o_raw_filepath(&self) -> &str { self.o_raw.to_str().unwrap() }
    //fn overwriting(&self) -> bool { self.n_raw_p || self.n_xmp_p }
    fn issues(&self) -> bool {
        self.n_raw_p || self.n_xmp_p || // overwriting
        self.timestamp.is_none()
    }
    fn n_raw(&self) -> Result<PathBuf,()>{
        match &self.timestamp {
            None => Err(()),
            Some(tstamp) => Ok(prefix_file(&self.o_raw, tstamp))
        }
    }
    fn o_xmp(&self) -> Result<PathBuf,()>{
        match self.o_xmp_p {
            false => Err(()),
            true => Ok(xmpize(&self.o_raw))
        }
    }
    fn n_xmp(&self) -> Result<PathBuf,()>{
        match self.n_raw() {
            Err(_) => {Err(())}
            Ok(new_raw) => {Ok(xmpize(&new_raw))}
        }
    }
    fn new(old_raw: &PathBuf, datestamp: bool) -> Self {
        let before = std::time::Instant::now();
        let o_xmp_p = xmpize(old_raw).exists() && xmpize(old_raw).is_file();
        let (n_raw_p, n_xmp_p): (bool,bool);
        let timestamp: Option<String> = match rexiv2::Metadata::new_from_path(old_raw) {
            Err(_) => None,
            Ok(mdata) => { match mdata.get_tag_string("Exif.Photo.DateTimeOriginal") {
                Err(_) => None,
                Ok(s) => {match datestamp { // "2023:04:02 12:45:10"
                    // "2023-04-02"
                    true => Some(format!("{}-{}-{}",     &s[0..4], &s[5..7], &s[8..10])),
                    // "2023-04-02-12:45:10"
                    false => Some(format!("{}-{}-{}-{}", &s[0..4], &s[5..7], &s[8..10], &s[11..]))
                }}
            }}
        };
        (n_raw_p, n_xmp_p) = match &timestamp {
            None => (false,false),
            Some(ts) => (prefix_file(old_raw, &ts).exists(),
                                xmpize(&prefix_file(old_raw, &ts)).exists())
        };
        ReplaceAction { o_raw: old_raw.to_path_buf(), 
            n_raw_p, timestamp, o_xmp_p, n_xmp_p, time: before.elapsed()}
        
    }
    fn run(&self){
        match self.issues() {
            true => {println!("Issues found, not running this.")}
            false => {
                _ = std::fs::rename(&self.o_raw,self.n_raw().unwrap());
                if self.o_xmp_p {
                    _ = std::fs::rename(self.o_xmp().unwrap(), self.n_xmp().unwrap());
                }
            }
        }
    }
}

impl fmt::Display for ReplaceAction{
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.timestamp {
            None => match self.o_raw_p() {
                true => write!(f,"{}{}","File not present: ".red(), self.o_raw_filepath()),
                false => write!(f,"{}{}","Could not parse: ".red(), self.o_raw_filepath()) }
            Some(tstamp) => {
                let new_filename_display = match self.n_raw_p{
                    true => format!("{}{}{}",tstamp,"_",filename_s(&self.o_raw)).red().to_string(),
                    false => format!("{}{}{}",tstamp.bright_green(),"_".green(),filename_s(&self.o_raw))
                };
                let xmp_display = match (self.o_xmp_p, self.n_xmp_p){
                    (false,false) => format!("{}","[.xmp]".hidden()), // happy path w/o XMP
                    (false,true)  => format!("[{}]",".xmp".dimmed().bold()), // "overwriting" (no source XMP)
                    (true,false)  => format!("[{}]",".xmp".green().bold()), // happy path w/XMP
                    (true,true)   => format!("[{}]",".xmp".red().bold()), // actual overwriting
                };

                if self.n_raw_p { write!(f,"{}{}{}", folder_s(&self.o_raw),new_filename_display, xmp_display )
                }else{            write!(f,"{}{}{}", folder_s(&self.o_raw),new_filename_display, xmp_display ) }
            }
        }
    }
}

fn prefix_file(f:&PathBuf,p:&str) -> PathBuf {
    f.with_file_name(format!("{}_{}",p,f.file_name().unwrap().to_str().unwrap())) }

fn xmpize(f:&PathBuf) -> PathBuf{
    f.with_file_name(format!("{}.xmp",f.file_name().unwrap().to_str().unwrap())) }

fn filename_s(f:&PathBuf) -> String{format!("{}",f.file_name().unwrap().to_str().unwrap())}
fn folder_s(f:&PathBuf) -> String{
    match f.parent(){
        None => format!(""),
        Some(p) => {
            if p.to_str().unwrap().is_empty(){
                format!("{}",p.to_str().unwrap())
            }else{
                format!("{}/",p.to_str().unwrap())
            }
        }
}}

fn is_not_xmp(f:&PathBuf) -> bool{
    // checkear que no arranque con <?xml
    match (f.extension(),f.exists()) {
        (_,false) => true,
        (None,true) => true,
        (Some(e),true) => e.to_ascii_lowercase() != "xmp"
    }
}

fn main() {
    let program_init = std::time::Instant::now();
    let mut args = Cli::parse();
    rexiv2::initialize().expect("Unable to initialize rexiv2");
    //match args.datestamp { true  => println!("# Use datestamps"), false => println!("# Use timestamps") }
    //match args.dryrun{ true  => println!("# Do a dry-run"), false => println!("# Run for real") }
    args.files.sort();
    // clean up XMP files, we're only looking for RAW files here
    args.files.retain(|x| is_not_xmp(&x));

    //let replace_actions: Vec<ReplaceAction> = c![ReplaceAction::new(filepath, args.datestamp), for filepath in &args.files];
    let replace_actions_start = std::time::Instant::now();
    let replace_actions: Vec<ReplaceAction> = args.files.par_iter().map(
        |fp|ReplaceAction::new(fp,args.datestamp)).collect();
    let replace_actions_time = replace_actions_start.elapsed();
    let issues = !c![a.issues(), for a in &replace_actions, if a.issues()].is_empty();
    //match issues{true => println!("There's issues =(\n"), false => println!("All OK =)\n") }
    match args.quiet{
        true => {if !issues && !args.dryrun{ for a in &replace_actions{a.run();} }}
        false  => {match (!issues && !args.dryrun,args.benchmark){
            (true,true) => { for a in &replace_actions{ // =) + b
                let a_time = std::time::Instant::now();
                a.run();
                println!("{} {:.2?}",a,a.time + a_time.elapsed());
            }}
            (true,false) => { for a in &replace_actions{ // =) + !b
                a.run(); println!("{}",a);
            }}
            (false,true) => { for a in &replace_actions{ // =( + b
                println!("{} {:.2?}",a,a.time);
            }}
            (false,false) => { for a in &replace_actions{ // =( + !b
                println!("{}",a);
            }}
        }}}
    
    if issues{println!("[{}]","Issues found".yellow());}
    if args.dryrun{println!("[{}]","Dry-run".purple());}
    println!("Total files: {}", replace_actions.len());
    if args.benchmark{
        println!("Average parsing time: {:.2?}", replace_actions_time / replace_actions.len() as u32);
        println!("Total parsing time: {:.2?}",replace_actions_time);
        println!("Total runtime: {:.2?}", program_init.elapsed()); 
    }
        
    //for a in replace_actions { println!("{}",a);}
    // if !args.dryrun {for a in replace_actions{ a.run(); }}
    /*
    for a in replace_actions {
        let orp = if a.o_raw_p() {"ORP"}else{"ORA"};
        let nrp = if a.n_raw_p{"NRP"}else{"NRA"};
        let oxp = if a.o_xmp_p{"OXP"}else{"OXA"};
        let nxp = if a.n_xmp_p{"NXP"}else{"NXA"};

        println!("{orp}|{nrp}|{oxp}|{nxp}|{a}");
    } // */
}