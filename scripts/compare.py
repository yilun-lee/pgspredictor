import pandas as pd
from typing import List
from plotnine import *

def read_plink_score(score_path: str, pgs: str):
    DA = pd.read_csv(score_path, sep = "\s+")
    DA = DA[["#FID", "IID", "SCORE1_SUM"]]
    DA.columns = ["FID", "IID", pgs]
    return DA
    
def read_pgspredictor_score(score_path: str, pgs: str):
    DA = pd.read_csv(score_path, sep = ",")
    DA = DA[["FID", "IID", pgs]]
    return DA
    


def run_one_pgs(pgs: str):
    plink_score_path = f"/yilun/test/pgspredict/{pgs}.sscore"
    pgspredictor_score_path = f"/yilun/test/pgspredict/{pgs}.score.csv"

    PLINK = read_plink_score(plink_score_path, pgs)
    DA = read_pgspredictor_score(pgspredictor_score_path, pgs)
    DA = DA.merge(PLINK, on = ["FID", "IID"], how = "inner", suffixes=["_my", "_plink"])
    COR = DA[[pgs+"_my", pgs+"_plink"]].corr(method="pearson")
    return float(COR.iloc[1,0])


def read_pgs_name(pgs_list_path: str) -> List[str]:
    pgs_list = []
    with open(pgs_list_path,"r") as FF:
        for i in FF.readlines():
            pgs_list.append(i.rstrip())
    
    return pgs_list
            
def plot_cor(DA: pd.DataFrame, save_path: str):
    p = (
        ggplot() +
        geom_point(data = DA, mapping = aes(x='PGS', y="CORR")) + 
        theme_classic() + 
        theme(
            axis_title = element_text(size=14),
            legend_title = element_text(size=14),
            axis_text = element_text(size=12),
            axis_text_x = element_text(rotation=90),
            legend_text = element_text(size=12),
            plot_title = element_text(size=17),
            strip_text = element_text(size=14),
        ) +
        labs(x = "PGS", y = "correlation") +
        guides(color = False)
    )
    p.save(filename = save_path , height = 6, width = 10, dpi = 300)




if __name__ == "__main__":
    pgs_list_path = "/mnt/prsdata/Test/TSGH_PGS_TWB2/pgs_list"
    pgs_list = read_pgs_name(pgs_list_path)
    
    LL = []
    for pgs in pgs_list:
        cor = run_one_pgs(pgs)
        LL.append({"PGS": pgs, "CORR": cor})
        
    DA = pd.DataFrame(LL)
    plot_cor(DA, "/yilun/CODE/pgspredictor/data/output/perf.png")

