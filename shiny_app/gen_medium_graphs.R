#! /usr/bin/Rscript

source("app.R");

ggsave(
    "awc_vs_reqwest.png",
    path = "../fixtures",
    width = 11,
    height = 6,
    plot_projects(
        stack_df(
            dfs[
                dfs$Project %in% c("actix_full", "actix_reqwest") &
                dfs$Optimization == "z" &
                dfs$LTO == "fat" &
                dfs$Codegen.Units == "1",
               ,],
            3
        )
    )
)

ggsave(
    "crates_dist.png",
    path = "../fixtures",
    width = 11,
    height = 4,
    plot_crates_box(
        dfs[
            dfs$Optimization == "z" &
            dfs$LTO == "fat" &
            dfs$Codegen.Units == "1",
           ,]
    )
)


ggsave(
    "optimization_tide_surf.png",
    path = "../fixtures",
    width = 11,
    height = 6,
    plot_optimization(
        dfs[
            dfs$Project == "tide_surf"
           ,]
    ) +
    labs(color = "LTO, Codegen Units")
)

ggsave(
    "projects_sizes.png",
    path = "../fixtures",
    width = 11,
    height = 6,
    plot_projects(
        stack_df(
            dfs[
                dfs$Project != "dummy" &
                dfs$Optimization == "z" &
                dfs$LTO == "fat" &
                dfs$Codegen.Units == "1",
               ,],
            0
        )
    ) +
    theme(legend.position = "none")
)
