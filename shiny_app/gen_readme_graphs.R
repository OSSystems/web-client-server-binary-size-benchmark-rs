#! /usr/bin/Rscript

require(svglite)
source("app.R");

ggsave(
    "actix_full_crates_size.svg",
    path = "../fixtures",
    width = 11,
    height = 4,
    plot_projects(
        stack_df(
            dfs[
                dfs$Project == "actix_full" &
                dfs$Optimization == "z" &
                dfs$LTO == "fat" &
                dfs$Codegen.Units == "1",
               ,],
            5
        )
    )
)

ggsave(
    "crates_dist.svg",
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
    "optimization_tide_surf.svg",
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
    "projects_sizes.svg",
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
