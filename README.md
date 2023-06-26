## Pgs-Predictor-rs

This is a bio-informatics tools to calculate polygenic risk score (PGS) from [pgs model weights](https://www.pgscatalog.org) and [plink bed file](https://www.cog-genomics.org/plink/1.9/formats#bed) format. The main puropose of this tool is just like [plink2 linear score](https://www.cog-genomics.org/plink/2.0/score). However, it is aimed to provided more functions than plink. For now, the tool can take care of snp matching by physical position and a1 allele, and report number of match snps. And there is also option to use provided frequency to fill missing. The tools is still working in progress.


### Feature

We often use [plink](https://www.cog-genomics.org/plink) to calculate pgs, which is very fast and handy. However, plink only take care of the prediction for one model with swap and missing strategy. For a pgs pipeline, we requested for more features. 

##### SNP joining and duplication handling

First is the join of snp between genotype file and weights. Plink only care about the snp id. Here, we used CHR and POS and A1 to match snp, so the uses don't need to care about them beforehand. Moreover, we also deal with duplicated snp, which plink often ignored or raised error. Unlike plink use column index to fetch ID, A1 and weights, user-defined column names are accepted.

##### Multiple models and percentils

There may be multiple pgs model for inference on the same genotype. **pgs-predictor-rs** can infer multiple models at the same times. In addition to scores, **pgs-predictor-rs** output percentils, rank and match status, which will be very beneficial in the downstream analyis. Last but not the least, user can provided score distribution from reference population, **pgs-predictor-rs** can calulate percentile of the score of the predicted against reference population (WIP feature).

##### Batch and multiprocessing

User can set batch size and number of thread for the program. Batch can be applied on sample axis or snp axis depending on your data. For genotype and weights that can fit into memory, multi-threading can help you to accelerate the whole program. For weights larger then memory, you can run batch along snp. Otherwise, for larger genotype, which is rare, you may run batch along sample. Multi-threading is still beneficial in such circumstance. **pgs-predictor-rs** can run faster (>5 times) than plink with proper combination of parametes even when there is only one model. 


### Pipeline

The following pipeline is how **pgs-predictor-rs** predict pgs from weights and genotype.

1. Read Bfile and Beta
2. Join Bfile and Beta (by CHR, POS, A1 or by ID, A1) and report match status.
3. Make predictions (There are three things doing here)
   1. Swap snp if needed.
   2. Fill missing genotype with strategy specified by user.
   3. Dot Genotypes and weights to get pgs score.
4. Save results and calculate rank and percentile if specified. 

### Usage

You may check the usage as following:

```bash
predictor -h
```

```console
A pgs predictor written in rust

Usage: pgspredictor [OPTIONS] --out-prefix <OUT_PREFIX> <WEIGHT_PATH> <BED_PATH>

Arguments:
  <WEIGHT_PATH>
          weight path, should be a tsv file
  <BED_PATH>
          path to plink bed files

Options:
  -o, --out-prefix <OUT_PREFIX>
          output prefix
  -n, --score-names <SCORE_NAMES>
          score names: scores to be process
  -T, --thread-num <THREAD_NUM>
          number of thread to run [default: 1]
  -B, --batch-size <BATCH_SIZE>
          batch size for sample / or snp if batch-snp flag is set [default: 10000]
      --match-id-flag
          whether to match by id instead of match by pos and chrom
  -v, --verbose...
          whether to show log, use -v -vv -vvv to present increase log level
      --batch-ind
          whether to batch by ind, default is batch by snp
      --chrom <CHROM>
          chromosome column for weight file [default: CHR]
      --pos <POS>
          position column for weight file [default: POS]
      --snp-id <SNP_ID>
          id column for weight file [default: ID]
      --a1 <A1>
          a1 column for weight file [default: A1]
      --freq <FREQ>
          freq column for weight file [default: FREQ]
      --pvalue <PVALUE>
          pvalue column for weight file, required when --q-ranges is specifeid [default: P]
  -M, --missing-strategy <MISSING_STRATEGY>
          Strategy to deal with missing value in genotype. Should be one of the following: Freq, Impute and Zero [default: Impute]
      --write-beta
          whether to write matched snp and related information to *.beta.csv
  -E, --eval-flag
          whether to calculate correlation between PHENO and score
  -P, --percentile-flag
          whether to output percentile and rank
  -R, --rank-path <RANK_PATH>
          path to rank file produce by pgs-predictor. RANK as the first column, which is 0~100, and the other column are score names. If specified, percentiles of sample scores are interpolated based on the rank
  -Q, --q-ranges <Q_RANGES>
          q range file, a headerless tsv file consisted of three columns: **name**, **from** and **to**, used in filtering p value for weights
  -h, --help
          Print help
  -V, --version
          Print version
```

There are three required arguments: `--weight-path <WEIGHT_PATH> --bed-path <BED_PATH> --out-path <OUT_PATH>` 

##### WEIGHT_PATH

model weights, used for predict pgs score. For more info you may check [pgs-catalog](https://www.pgscatalog.org). A tab-deliminated (tsv) file which consists of the following columns:

- chrom: **string**, chromosome index. You can specify the column name by `--chrom`.
- pos: **int**, physical position of snp. Specify the column name by `--pos`.
- snp-id: **string**, snp identifier, *optional*, needed only when `--match-id-flag` is specified. Specify the column name by `--snp-id`.
- a1: **string**, effected allele for weight. Specify the column name by `--a1`.
- score-names: **float**, the weights of different algorithm. There can be multiple score name columns. You may specify them with flag like: `-n Lassosum -n LDpred2 -n CandT`.
- freq: **float**, allele frequency. *optional* but recommended. Only needed when `--missing-strategy` is `Freq`. Missing value will be filled with the corresponding frequency. Please noted that the frequency should belong to a1 allele in the same file. Specify the column name by `--freq`.

The order of the above columns can be arbitary. Other columns in the tsv will be ignored without causing any problem. 

This is an [example](./data/input/Weights.tsv) of a beta file with two prs algo [CandT](https://www.biorxiv.org/content/10.1101/653204v2.full) and [Lassosum](https://github.com/tshmak/lassosum) and many other columns:

```console
CHR	POS	ID	REF	ALT	A1	Lassosum	FREQ	CandT
2	27508073	rs1260326	T	C	C	0.03669449	0.2	0.02
2	27519736	rs780093	T	C	C	0.001817981	0.01	-0.006
```

##### BED_PATH

[Plink bed file](https://www.cog-genomics.org/plink/1.9/formats#bed) format is a binary and perfomant data format storing genotype data. It is a triplet: fam, bim and [bed](https://www.cog-genomics.org/plink/2.0/input). fam file is the metadata for sample, such as sample id, phenotype and sex. bim is the metadata for snp, such as chrom, snp id. bed is the binary file containing the whole genotype matrix. The three files should share the same prefix, and `--bed-path` accept that prefix. Example files are in the folder [here](./data/input).

##### OUT_PATH

This argument (`--out-path`) is the output prefix. For now, there are two output files: `{out_path}.check.json` and `{out_path}.score.csv`. The json recording the bfile snp number, model snp number and match snp number. The csv containing the predicted score for each individual. Example files are [here](./data/output/test.check.json) and [here](./data/output/test.score.csv). If `-P` or `--percentile-flag` is specified, two additional files will be produced: `{out_path}.percentiles.csv` and `{out_path}.rank.csv`. Example files are in the same [folder](./data/output/) `{out_path}.percentiles.csv` is the percentiles for each sample from the predicted population or reference popluation if score distribution, or rank, from reference popluation is provided. Rank is the 0-100 quantils for the score distribtuion, used as refernce for other model to make predictions (WIP).

```bash
cat ${out_path}.check.json
```

```console
{
  "bfile_snp": 103894,
  "match_snp": 2,
  "model_snp": 2
}
```

```bash
cat ${out_path}.score.csv
```

```console
FID,IID,Lassosum,CandT
sim_000HDES,sim_000HDES,0.077024944,0.027999999
sim_0033NJR,sim_0033NJR,0.0,0.0
sim_00SMPKV,sim_00SMPKV,0.038512472,0.0139999995
sim_00Z7G8G,sim_00Z7G8G,0.077024944,0.027999999
sim_02E7H7D,sim_02E7H7D,0.0,0.0
sim_02J2ENE,sim_02J2ENE,0.038512472,0.0139999995
sim_02V9A69,sim_02V9A69,0.0,0.0
sim_03D1RGH,sim_03D1RGH,0.0,0.0
sim_03JCPNG,sim_03JCPNG,0.0,0.0
```

##### missing strategy 

User can specifeid how program handle missing genotype through `-M` or `--missing-strategy` flag. There are three strategy for now:
1. **Freq**: Use frequency column in weights (`FREQ` as default, can be specified by `--freq`) to fill the misisng.  This strategy is recommended for prediction. 
2. **Impute**: Impute frequency of the current population to fill the misisng. Recommended for validation. Not recommended for small population. 
3. **Zero**: Fill missing with zeors. Not recommended.



##### Quick Example

This is a simple exaple using very small bfile and weights. You may check the output files in [data/output](./data/output/) 

```bash
./target/release/predictor \
    -m "data/input/Weights.tsv" \
    -b "data/input/test" \
    -o "data/output/test" \
    -T 1 -B 5 \
    -n "Lassosum" -n CandT -M Impute
```


### Todo

1. Validation, the result should be better than plink2. 
2. Validation and Prediction mode
3. Integrated with pgs catalog.
4. Support for more genotype format.
5. Improve and benchmark speed.
6. CICD for testing and distributing


### Install

Currently, use cargo build.
The development environment os on mac, so the [blas-src](https://github.com/blas-lapack-rs/blas-src) is **accelerate**, feel free to change it in [reader cargo.toml](./reader/Cargo.toml). and [predictor cargo.toml](./predictor/Cargo.toml).

```bash 
# clone
git clone https://github.com/yilun-lee/pgspredictor.git
cd pgspredictor
# build
cargo build -p predictor -r
# run
./target/release/predictor -h
```

