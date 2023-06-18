## Prs-Predictor-rs

This is a bio-informatics tools to calculate polygenic risk score (PGS) from [pgs model weights](https://www.pgscatalog.org) and [plink bed file](https://www.cog-genomics.org/plink/1.9/formats#bed) format. The main puropose of this tool is just like [plink2 linear score](https://www.cog-genomics.org/plink/2.0/score). However, it is aimed to provided more functions than plink. For now, the tool can take care of snp matching by physical position and a1 allele, and report number of match snps. And there is also option to use provided frequency to fill missing. The tools is still working in progress.

### Usage

You may check the usage as following:

```bash
predictor -h
```

```console
A pgs predictor written in rust

Usage: predictor [OPTIONS] --weight-path <WEIGHT_PATH> --bed-path <BED_PATH> --out-path <OUT_PATH>

Options:
  -m, --weight-path <WEIGHT_PATH>
          weight path, should be a tsv file
  -b, --bed-path <BED_PATH>
          path to plink bed files
  -o, --out-path <OUT_PATH>
          output prefix
  -n, --score-names <SCORE_NAMES>
          score names: scores to be process
  -T, --thread-num <THREAD_NUM>
          number of thread to run [default: 1]
  -B, --batch-size <BATCH_SIZE>
          batch size for sample [default: 10000]
  -F, --freq-flag
          Use freq to fill missing or not
      --match-id-flag
          if match by id
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
- freq: **float**, allele frequency. *optional* but recommended. Only needed when `--freq-flag` is specified. Missing value will be filled with the corresponding frequency. Please noted that the frequency should belong to a1 allele in the same file. Specify the column name by `--freq`.

The order of the above columns can be arbitary. Other columns in the tsv will be ignored without causing any problem.

This is an example of a beta file with two prs algo [CandT](https://www.biorxiv.org/content/10.1101/653204v2.full) and [Lassosum](https://github.com/tshmak/lassosum) and many other columns:

```console
CHR	POS	ID	REF	ALT	A1	Lassosum	FREQ	CandT
2	27508073	rs1260326	T	C	C	0.03669449	0.2	0.02
2	27519736	rs780093	T	C	C	0.001817981	0.01	-0.006
```

##### BED_PATH

[Plink bed file](https://www.cog-genomics.org/plink/1.9/formats#bed) format is a binary and perfomant data format storing genotype data. It is a triplet: fam, bim and [bed](https://www.cog-genomics.org/plink/2.0/input). fam file is the metadata for sample, such as sample id, phenotype and sex. bim is the metadata for snp, such as chrom, snp id. bed is the binary file containing the whole genotype matrix. The three files should share the same prefix, and `--bed-path` accept that prefix.

##### OUT_PATH

This argument (`--out-path`) is the output prefix. For now, there are two output files: `{out_path}.check.json` and `{out_path}.score.csv`. The json recording the bfile snp number, model snp number and match snp number. The csv containing the predicted score for each individual. 

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




### Install

Currently, use cargo build.
The development environment os on mac, so the blas-src is **accelerate**, feel free to change it in [reader cargo.toml](./reader/Cargo.toml). and [predictor cargo.toml](./predictor/Cargo.toml).

```bash 
# clone
git clone https://github.com/yilun-lee/pgspredictor.git
cd pgspredictor
# build
cargo build -p predictor -r
# run
./target/release/predictor -h
```

