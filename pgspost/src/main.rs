//mod covariate;
mod post;
mod read_score;
mod args;

use args::{MyArgs, match_log};
use clap::Parser;
use post::PgsPost;
use read_score::PgsScores;


fn main() {
    // parse input
    let mut cli: MyArgs = MyArgs::parse();
    match_log(cli.verbose);
    cli.check_defaul().unwrap();

    // read score
    let scores = PgsScores::read_score(&cli.score_path, &cli.score_names).unwrap();
    let mut pgs_post = PgsPost::new(&scores, &cli.out_prefix, &cli.rank_path, cli.eval_flag);
    pgs_post.write_output().unwrap();
}


