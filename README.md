## Pgs-Predictor-rs

This is a bio-informatics tools to calculate polygenic risk score (PGS) from [pgs model weights](https://www.pgscatalog.org) and [plink bed file](https://www.cog-genomics.org/plink/1.9/formats#bed) format. The main puropose of this tool is just like [plink2 linear score](https://www.cog-genomics.org/plink/2.0/score). However, it is aimed to provided more functions than plink. For now, the tool can take care of snp matching by physical position and a1 allele, and report number of match snps. And there is also option to use provided frequency to fill missing. The tools is still working in progress. There are two binary in this report: **pgspredictor** and **pgspost**. **pgspredictor** is the classic usecase for prediction, while **pgspost** is the post analysis based on predictions, such as percentile, evaluation and covariate.


### Feature

We often use [plink](https://www.cog-genomics.org/plink) to calculate pgs, which is very fast and handy. However, plink only take care of the prediction for one model with swap and missing strategy. For a pgs pipeline, we requested for more features. In addition, we also add q-range support for doing CandT like plink.

##### SNP joining and duplication handling

First is the join of snp between genotype file and weights. Plink only care about the snp id. Here, we used CHR and POS and A1 to match snp, so the uses don't need to care about them beforehand. Moreover, we also deal with duplicated snp, which plink often ignored or raised error. Unlike plink use column index to fetch ID, A1 and weights, user-defined column names are accepted.

##### Multiple models and percentils, covariates

There may be multiple pgs model for inference on the same genotype. **pgs-predictor-rs** can infer multiple models at the same times. In addition to scores, **pgs-predictor-rs** output percentils, rank and match status, which will be very beneficial in the downstream analyis. Last but not the least, user can provided score distribution from reference population, **pgs-predictor-rs** can calulate percentile of the score of the predicted against reference population. We also working on to add simple covariate support. 

##### Batch and multiprocessing

User can set batch size and number of thread for the program. Batch can be applied on sample axis or snp axis depending on your data. For genotype and weights that can fit into memory, multi-threading can help you to accelerate the whole program. For weights larger then memory, you can run batch along snp. Otherwise, for larger genotype, which is rare, you may run batch along sample. Multi-threading is still beneficial in such circumstance. For speed, **pgs-predictor-rs** is a bit slower than plink with proper combination of parametes.



### Pipeline

The following pipeline is how **pgspredictor**  predict pgs from weights and genotype.

1. Read Bfile and Beta
2. Join Bfile and Beta (by CHR, POS, A1 or by ID, A1) and report match status.
3. Make predictions (There are three things doing here)
   1. Swap snp if needed.
   2. Fill missing genotype with strategy specified by user.
   3. Dot Genotypes and weights to get pgs score.
4. Save predict results. 

After prediction, **pgspost** may do the following (WIP): 

1. Load score, Load covariate (optional)
2. Fit or predict with covariate model (optional, WIP)
3. Convert it to percentile w/wo reference rank.
4. Evaluate if specified.
5. Save retults 


### Usage

You may check the usage as following. Only show the tupper part. For detail feel free to run the help command yourself.

#### pgspredictor

```bash
pgspredictor -h
```

```console
Usage: pgspredictor [OPTIONS] --out-prefix <OUT_PREFIX> <MODE> <WEIGHT_PATH> <BED_PATH>

Arguments:
  <MODE>
          analysis mode, one of ["Validate", "Predict", "Run", "CandT"]
  <WEIGHT_PATH>
          weight path, should be a tsv file
  <BED_PATH>
          path to plink bed files
```

Because there are a lot of argument, we add some shortcut, that is, different mode, to use preset parameter. Please check the table. For **Run** mode, no argument check is done.

|Mode|--write-beta|--missing-strategy|--q-ranges|
| :--: | :--: | :--: | :--: |
|Validate|true|Impute|X|
|CandT|true|Impute|V|
|Predict|false|Freq|X|
|Run|-|-|-|

#### pgspost

Same as **pgspredictor**. There are mode as shortcut and check.

```bash
pgspost -h
```

```console
Usage: pgspost [OPTIONS] --out-prefix <OUT_PREFIX> <MODE> <SCORE_PATH>

Arguments:
  <MODE>
          mode, should be Predict / Validate
  <SCORE_PATH>
          weight path, should be a tsv file
```
|Mode|--eval-flag|--rank-path|
| :--: | :--: | :--: |
|Validate|true|X|
|Predict|false|V|
|Run|-|-|



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

