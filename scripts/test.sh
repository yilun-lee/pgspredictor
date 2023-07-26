
PLINK2="plink2 --bfile /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG --score /Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv 3 6 7 header cols=+scoresums --out test"
RUST_PRS="./target/release/predictor"
hyperfine --warmup 3 -r 10 "${PLINK2}" "${RUST_PRS}"

export RUST_BACKTRACE=1
cargo build -p pgspredictor
./target/debug/pgspredictor \
    Predict \
    "data/input/Weights.tsv" \
    "data/input/test" \
    -o "data/output/test" \
    -T 1 -B 5 \
    -n "Lassosum" -n CandT -M Impute -P \
    --rank-path data/output/test.rank.csv -vv 


BFILE="/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/rename"
MODEL="/Users/sox/Desktop/AILAB_DATA/Data/PGS000099.tsv"
./target/debug/pgspredictor \
    Validate \
    ${MODEL} \
    ${BFILE} \
    -o /tmp/test \
    -T 1 -B 10000 \
    -n PGS000099  -M Impute -E -vv --batch-ind


cargo build -p pgspredictor -r 

BFILE="/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/rename"
MODEL="/Users/sox/Desktop/AILAB_DATA/Data/PGS000099.tsv"
./target/release/pgspredictor \
    ${MODEL} \
    ${BFILE} \
    -o /tmp/test \
    -T 1 -B 10000 \
    -n PGS000099  -M Impute -E 


hyperfine --warmup 3 -r 10 \
    "./target/release/pgspredictor \
    ${MODEL} \
    ${BFILE} \
    -o /tmp/test \
    -T 1 -B 20000 \
    -n PGS000099  -M Impute 
    " \
    "./target/release/pgspredictor \
    ${MODEL} \
    ${BFILE} \
    -o /tmp/test \
    -T 1 -B 20000 \
    -n PGS000099  -M Impute  --batch-ind
    " \
    "plink2 --bfile ${BFILE}  \
    --score ${MODEL} 3 6 10 header cols=+scoresums ignore-dup-ids\
    --out /tmp/test"



./target/debug/pgspost \
    Validate \
    "data/output/test.score.csv" \
    -o "data/output/test" \
    -n "Lassosum" -n CandT -vv -E


./target/debug/pgspost \
    Predict \
    "data/output/test.score.csv" \
    -R data/output/test.rank.csv \
    -o "data/output/test" \
    -n "Lassosum" -n CandT -vv 


