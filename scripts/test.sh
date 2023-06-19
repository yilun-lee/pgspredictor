
PLINK2="plink2 --bfile /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG --score /Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv 3 6 7 header cols=+scoresums --out test"
RUST_PRS="./target/release/predictor"
hyperfine --warmup 3 -r 10 "${PLINK2}" "${RUST_PRS}"

export RUST_BACKTRACE=1
cargo build -p predictor
./target/release/predictor \
    -m "data/input/Weights.tsv" \
    -b "data/input/test" \
    -o "data/output/test" \
    -T 1 -B 10 \
    -n "Lassosum" -n CandT -F 

