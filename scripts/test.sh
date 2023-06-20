
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
    -n "Lassosum" -n CandT -M Zero



hyperfine --warmup 3 -r 10 " ./target/release/predictor \
    -m /Users/sox/Desktop/AILAB_DATA/Data/model.tsv \
    -b /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/rename \
    -o /tmp/test \
    -T 4 -B 2000 \
    -n PGS000099  -M Impute " \
    " ./target/release/predictor \
    -m /Users/sox/Desktop/AILAB_DATA/Data/model.tsv \
    -b /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/rename \
    -o /tmp/test \
    -T 4 -B 2000 \
    -n PGS000099  -M Zero " \
    "plink2 --bfile /Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/rename \
    --score /Users/sox/Desktop/AILAB_DATA/Data/model.tsv 3 6 10 header cols=+scoresums \
    --out /tmp/test"


