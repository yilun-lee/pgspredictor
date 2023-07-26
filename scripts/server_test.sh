


cd /volume/prsdata/Users/yilun/CODE/pgspredictor

OUTPATH="/yilun/test/pgspredict/test"
BFILE="/yilun/test/pgspredict/TWB2.rename"
MODEL="/mnt/prsdata/Test/TSGH_PGS_TWB2/PGS002742/models/PGS002742/model.tsv"
/usr/bin/time -v  ./target/release/pgspredictor \
    Validate \
    ${MODEL} \
    ${BFILE} \
    -o ${OUTPATH} \
    -T 8 -B 40000 \
    -n PGS002742  -M Impute  -vv


/usr/bin/time -v plink2 --bfile ${BFILE}  --threads 8 \
    --score ${MODEL} 3 6 10 header cols=+scoresums ignore-dup-ids\
    --out ${OUTPATH} 



/mnt/prsdata/Test/TSGH_PGS_TWB2/PGS001248/models/PGS001248/model.tsv
wc /mnt/prsdata/Test/TSGH_PGS_TWB2/*/models/*/model.tsv



for i in $(cat /mnt/prsdata/Test/TSGH_PGS_TWB2/pgs_list ); do

OUTPATH="/yilun/test/pgspredict/${i}"
BFILE="/yilun/test/pgspredict/TWB2.rename"
MODEL="/mnt/prsdata/Test/TSGH_PGS_TWB2/${i}/models/${i}/model.tsv"
/usr/bin/time -v  ./target/release/pgspredictor \
    Validate \
    ${MODEL} \
    ${BFILE} \
    -o ${OUTPATH} \
    -T 8 -B 20000 \
    -n ${i}  -M Impute  > ${OUTPATH}.pgspredictor.log 2>&1
done

/usr/bin/time -v plink2 --bfile ${BFILE}  --threads 8 \
    --score ${MODEL} 3 6 10 header cols=+scoresums ignore-dup-ids\
    --out ${OUTPATH}  > ${OUTPATH}.plink2.log 2>&1

done





# test ukb
OUTPATH="/yilun/test/pgspredict/UKB.PGS000331"
BFILE="/volume/prsdata/Genotype/UKB/ARRAY_ALL/UKB"
MODEL="/mnt/prsdata/Test/TSGH_PGS_TWB2/PGS000331/models/PGS000331/model.tsv"
/usr/bin/time -v  ./target/release/pgspredictor \
    Validate \
    ${MODEL} \
    ${BFILE} \
    -o ${OUTPATH} \
    -T 8 -B 20000 \
    -n PGS000331  -M Impute  -vv


/usr/bin/time -v plink2 --bfile ${BFILE}  --threads 8 \
    --score ${MODEL} 3 6 10 header cols=+scoresums ignore-dup-ids\
    --out ${OUTPATH} 


