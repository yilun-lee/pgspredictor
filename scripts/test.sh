
PLINK2="plink2 --bfile /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG --score /Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv 3 6 7 header cols=+scoresums --out test"
RUST_PRS="./target/release/predictor"
hyperfine --warmup 3 -r 10 "${PLINK2}" "${RUST_PRS}"

export RUST_BACKTRACE=1
cargo build -p predictor
./target/debug/predictor \
    -m "data/input/Weights.tsv" \
    -b "data/input/test" \
    -o "data/output/test" \
    -T 1 -B 5 \
    -n "Lassosum" -n CandT -M Impute -P \
    --batch-snp --rank-path data/output/test.rank.csv


BFILE="/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/rename"
MODEL="/Users/sox/Desktop/AILAB_DATA/Data/PGS000099.tsv"

cargo build -p predictor -r 
./target/debug/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 4 -B 2000 \
    -n PGS000099  -M Impute 

hyperfine --warmup 3 -r 10 \
    "./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 6 -B 2000 \
    -n PGS000099  -M Impute --batch-snp
    " \
    "./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 1 -B 16000 \
    -n PGS000099  -M Impute --batch-snp
    " \
    " ./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 6 -B 2000 \
    -n PGS000099  -M Impute " \
    " ./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 1 -B 16000 \
    -n PGS000099  -M Impute " \
    "plink2 --bfile ${BFILE} \
    --score ${MODEL} 3 6 10 header cols=+scoresums ignore-dup-ids\
    --out /tmp/test"



hyperfine --warmup 3 -r 10 \
    "./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 1 -B 13000 \
    -n PGS000099  -M Impute --batch-snp
    " \
    "./target/release/predictor \
    -m ${MODEL} \
    -b ${BFILE} \
    -o /tmp/test \
    -T 6 -B 2000 \
    -n PGS000099  -M Impute --batch-snp --percentile-flag
    " 

