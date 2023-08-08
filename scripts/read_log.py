import re
import pandas as pd
import numpy as np
from plotnine import *


def read_pgs_name(pgs_list_path: str):
    pgs_list = []
    with open(pgs_list_path,"r") as FF:
        for i in FF.readlines():
            pgs_list.append(i.rstrip())
    
    return pgs_list
            

def read_snp_num():
    DA = pd.read_csv("/yilun/test/pgspredict/PGS.snp.txt", header = None, sep = "\s+")
    DA.columns = ["snp_num", "model_path"]
    DA = DA.loc[DA.model_path!="total"]
    DA["pgs"] = DA.model_path.str.extract(r"(PGS[0-9]+)")
    DA.snp_num = DA.snp_num.astype(int)
    DA = DA[["pgs", "snp_num"]]
    return DA

def strtime_int(times: str):
    time_list = times.split(":")
    
    ll = len(time_list)
    
    total_sec = 0.
    mult = 1.
    for i in range((ll-1),-1,-1):
        total_sec += float(time_list[i]) * mult
        mult *= 60
    
    return total_sec

def parse_log(log_path: str):
    logs = []
    with open(log_path,"r") as FF:
        for i in FF.readlines():
            logs.append(i.rstrip())
    
    for line in logs:
        if "Elapsed" in line:
            times = re.search(r"Elapsed\s\(wall\sclock\)\stime\s\(h:mm:ss\sor\sm:ss\):\s([0-9:]+)", line).group(1)
            secs = strtime_int(times)
        if "Maximum" in line:
            memory = re.search(r"Maximum resident set size \(kbytes\): ([0-9]+)", line).group(1)
            memory = float(memory)/1024/1024

    return memory, secs


            
def plot_mem(DA: pd.DataFrame, save_path: str):
    p = (
        ggplot() +
        geom_point(data = DA, mapping = aes(x='snp_num', y="memory", color = "met")) + 
        theme_classic() + 
        theme(
            axis_title = element_text(size=14),
            legend_title = element_text(size=14),
            axis_text = element_text(size=12),
            legend_text = element_text(size=12),
            plot_title = element_text(size=17),
            strip_text = element_text(size=14),
        ) +
        labs(x = "log10 of model snp number", y = "memory (Gb)", color= "method")
    )
    p.save(filename = save_path , height = 6, width = 10, dpi = 300)

def plot_times(DA: pd.DataFrame, save_path: str):
    p = (
        ggplot() +
        geom_point(data = DA, mapping = aes(x='snp_num', y="secs", color = "met")) + 
        theme_classic() + 
        theme(
            axis_title = element_text(size=14),
            legend_title = element_text(size=14),
            axis_text = element_text(size=12),
            legend_text = element_text(size=12),
            plot_title = element_text(size=17),
            strip_text = element_text(size=14),
        ) +
        labs(x = "log10 of model snp number", y = "time (secs)", color= "method")
    )
    p.save(filename = save_path , height = 6, width = 10, dpi = 300)






if __name__ == "__main__":
    SNP = read_snp_num()
    pgs_list_path = "/mnt/prsdata/Test/TSGH_PGS_TWB2/pgs_list"
    pgs_list = read_pgs_name(pgs_list_path)

    LL = []
    for pgs in pgs_list:
        for met in ["plink2", 'pgspredictor']:
            log_path = f"/yilun/test/pgspredict/{pgs}.{met}.log"
            memory, secs = parse_log(log_path)
            LL.append({
                "pgs": pgs,
                "met": met,
                "memory": memory,
                "secs": secs,
            })
    
    DA = pd.DataFrame(LL)
    DA = DA.merge(SNP, on = "pgs")
    DA.snp_num = np.log10(DA.snp_num)
    plot_mem(DA, "/yilun/CODE/pgspredictor/data/output/mem.png")
    plot_times(DA, "/yilun/CODE/pgspredictor/data/output/time.png")

