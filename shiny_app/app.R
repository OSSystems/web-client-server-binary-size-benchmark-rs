#! /usr/bin/Rscript

library(rjson)
library(ggplot2)
library(shiny)
library(shinydashboard)
library(DT)
library(plotly)
library(dplyr)

display <- function(vs) {
  lapply(vs, function(v) {
    if (is.na(v)) return(v)
    if (v >= 1000000) {
      n <- v / 1000000
      paste0(round(n, 2), "M")
    } else if (v >= 1000) {
      n <- v / 1000
      paste0(round(n, 2), "K")
    } else {
      paste0(v)
    }
  })
}

gen_dataframe <- function(file) {
  data <- fromJSON(file = file)$crates
  len <- length(data)
  crates <- vector()
  sizes <- vector()
  for (i in 1:len) {
    crate <- data[[i]]
    crates[i] <- crate[[1]]
    sizes[i] <- crate[[2]]
  }
  data.frame(
      Crate = crates,
      Size = sizes,
      Project = sub("_opt_.*$", "", basename(file)),
      Optimization = sub(".*_opt_([0-9,s,z]*)_.*", "\\1", file),
      LTO = sub(".*_lto_([a-z]*)_.*", "\\1", file),
      Codegen.Units = sub(".*_cg_([0-9]*)$", "\\1", file),
      Relative.Size = ((sizes / sum(sizes)) * 100) %>% round(digits=2),
      Total = sum(sizes)
  )
}

stack_df <- function(set, threshold) {
  if (is.null(set)) return (NULL)
  n_df <- data.frame()
  for (p in unique(set$Project)) {
    df <- set[set$Project == p,]
    df <- df[order(df$Size, decreasing = TRUE),]
    if (threshold > 0) {
      n_df <- rbind(n_df, df[0:threshold,])
      if (threshold >= nrow(df)) next
      stack <- sum(df[-threshold:0,2])
      relative_stack <- sum(df[-threshold:0,7])
    } else {
      stack <- sum(df[,2])
      relative_stack <- 100
    }
    n_df <- rbind(
        n_df,
        list(
            "[Others]",
            stack,
            df[1,3],
            df[1,4],
            df[1,5],
            df[1,6],
            relative_stack,
            df[1,8]
        )
    )
  }
  names(n_df) <- names(set)
  n_df
}

plot_projects <- function(set) {
  if (is.null(set)) return(NULL)

  ggplot(set, aes(x = Project)) +
    scale_fill_discrete(name = "Crate") +
    scale_y_continuous(name = "Size", labels = display) +
    geom_text(
        aes(
            label = display(Total),
            y = Total+(max(Total) * 0.05)
        )
    ) +
    geom_col(
        aes(
            y = Size,
            fill = Crate
        )
    ) +
    coord_flip() +
    theme(text = element_text(size = 24))
}

plot_crates <- function(set, title = NULL) {
  if (is.null(set)) return(NULL)

  ggplot(set, aes(x = reorder(Crate, Size))) +
    scale_x_discrete(name = "Crate") +
    scale_y_continuous(name = "Size", labels = display) +
    geom_text(
        aes(
            label = display(Size),
            y = Size+(max(Size) * 0.03)
        )
    ) +
    geom_col(
        aes(
            y = Size,
            fill = TRUE
        )
    ) +
    coord_flip() +
    labs (title = title) +
    theme(
        text = element_text(size = 24),
        legend.position = "none"
    )
}

plot_crates_box <- function(set) {
  if (is.null(set)) return(NULL)

  ggplot(
      set,
      aes(
          x = "[All Crates]",
          y = Size,
          fill = TRUE
      )
  ) +
    scale_y_continuous(name = "Size", labels = display) +
    geom_boxplot() +
    coord_flip() +
    theme(
        text = element_text(size = 24),
        legend.position = "none",
        axis.title.y = element_blank(),
        axis.ticks.y = element_blank(),
        panel.grid.major.y = element_blank(),
        panel.grid.minor.y = element_blank()
    )
}

plot_crates_dist <- function(set, x) {
  if (is.null(set)) return (NULL)
  if(nrow(set) == 0) return (NULL)

  ggplot(set, aes(x = Crate, fill = Project)) +
    scale_y_continuous(name = "Size", labels = display) +
    scale_x_discrete(limits = x, drop = FALSE) +
    geom_col(
        aes(y = Size),
        position = "dodge"
    ) +
    geom_text(
        aes(
            y = Size+(max(Size) * 0.03),
            label = display(Size)
        ),
        position = position_dodge(0.9)
    ) +
    theme(
        text = element_text(size = 24)
    )
}

plot_optimization <- function(set) {
  if (is.null(set)) return(NULL)

  set <- set[!duplicated(set$Total), ]

  ggplot(
      set,
      aes(
          x = Optimization,
          y = Total,
          group = paste(LTO, Codegen.Units)
      )
  ) +
    scale_y_continuous(name = "Size", labels = display) +
    geom_point(
        aes(color = paste(LTO, Codegen.Units)),
        size = 3,
        ) +
    geom_line(
        aes(color = paste(LTO, Codegen.Units)),
        linetype = 2,
        ) +
    theme(
        text = element_text(size = 24)
    )
}

plot_codegen <- function(set) {
  if (is.null(set)) return(NULL)

  set <- set[!duplicated(set$Total), ]

  ggplot(
      set,
      aes(
          x = Project,
          y = Total,
          fill = Codegen.Units
      )
  ) +
    scale_y_continuous(labels = display) +
    scale_colour_manual(values = c("red", "black")) +
    geom_col(
        position = position_dodge2(
            paddin = 0.2
        ),
        size = 0.4
    ) +
    geom_text(
        aes(
            label = display(Total),
            y=Total+(max(Total) * 0.05)
        ),
        position = position_dodge(0.9)
    ) +
    coord_flip() +
    theme(
        text = element_text(size = 24)
    )
}

gen_table <- function(set) {
  df <- data.frame(Crate = unique(dfs$Crate))
  for (p in unique(set$Project)) {
    dfp <- dfs[dfs$Project == p,]
    dfp <- data.frame(x = dfp$Crate, y = dfp$Size)
    names(dfp) <- c("Crate", p)
    df <- merge(df, dfp, by = "Crate", all.x = TRUE)
  }
  df
}

files <- list.files(path = "results", full.names = TRUE)
dfs <- Reduce(function(x, y) { merge(x, y, all = TRUE) }, lapply(files, gen_dataframe))

ui <- dashboardPage(
    title = "Rust Server-Client Binary Size Benchmark",

    dashboardHeader(
        title = "Binary-Size Benchmark", titleWidth = 270
    ),

    dashboardSidebar(
        width = 270,
        sidebarMenu(
            id = "menu",
            menuItem(
                text = "Introduction",
                tabName = "info",
                icon = icon("info-circle")
            ),
            menuItem(
                text = "Projects Data",
                tabName = "projects",
                icon = icon("dashboard")
            ),
            menuItem(
                text = "Crates Data",
                tabName = "crates",
                icon = icon("cubes")
            ),
            menuItem(
                text = "Optimization Flags",
                tabName = "optimization",
                icon = icon("sort-amount-up")
            ),
            menuItem(
                text = "Full Data",
                tabName = "raw",
                icon = icon("table")
            )
        )
    ),

    dashboardBody(
        tags$head(tags$style(HTML(".main-sidebar { font-size: 20px; }"))),
        tabItems(
            tabItem(
                tabName = "info",
                fluidRow(
                    box(
                        title = "Geral Info",
                        width = 12, solidHeader = TRUE, status = "primary",
                        htmlOutput("t0_general")
                    )
                ),
                tags$head(tags$style("#t0_general{font-size: 24px;}"))
            ),
            tabItem(
                tabName = "projects",
                tags$head(tags$style(HTML('body {overflow-y: scroll;}'))),
                fluidRow(
                    box(
                        title = "Binary Size", width = 9, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t1_project_plot", height = 600)
                    ),
                    box(
                        title = "Controls", width = 3, solidHeader = TRUE, status = "primary",
                        selectInput(
                            "t1_projects_control",
                            "Projects:",
                            selected = unique(dfs$Project),
                            choices = unique(dfs$Project),
                            multiple = TRUE
                        ),
                        sliderInput(
                            "t1_stack_control", "Unstacked itens:",
                            min = 0, max = 10, value = 5
                        ),
                        selectInput(
                            "t1_opt_control",
                            "Optimization:",
                            selected = "0",
                            choices = unique(dfs$Optimization)
                        ),
                        selectInput(
                            "t1_lto_control",
                            "LTO:",
                            selected = "thin",
                            choices = unique(dfs$LTO)
                        ),
                        selectInput(
                            "t1_cu_control",
                            "Codegen Units:",
                            selected = "16",
                            choices = unique(dfs$Codegen.Units)
                        )
                    )
                ),
                fluidRow(
                    box(
                        title = "Detailed Crates for Project",
                        width = 12, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t1_crates_plot", height = "100%", width = "auto")
                    )
                )
            ),
            tabItem(
                tabName = "crates",
                fluidRow(
                    box(
                        title = "All Crates Distribution",
                        width = 12, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t2_box_plot", height = "600")
                    )
                ),
                fluidRow(
                    box(
                        title = "Specific Crates Distribution",
                        width = 9, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t2_crates_plot", height = "600")
                    ),
                    box(
                        title = "Controls", width = 3, solidHeader = TRUE, status = "primary",
                        selectInput(
                            "t2_crates_control",
                            "Crate:",
                            selected = c("std", "tokio"),
                            choice = unique(dfs$Crate),
                            multiple = TRUE
                        ),
                        selectInput(
                            "t2_opt_control",
                            "Optimization:",
                            selected = "0",
                            choices = unique(dfs$Optimization)
                        ),
                        selectInput(
                            "t2_lto_control",
                            "LTO:",
                            selected = "thin",
                            choices = unique(dfs$LTO)
                        ),
                        selectInput(
                            "t2_cu_control",
                            "Codegen Units:",
                            selected = "16",
                            choices = unique(dfs$Codegen.Units)
                        )
                    )
                )
            ),
            tabItem(
                tabName = "optimization",
                fluidRow(
                    box(
                        title = "Optimization",
                        width = 9, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t3_optimization_plot", height = "700")
                    ),
                    box(
                        title = "Controls", width = 3, solidHeader = TRUE, status = "primary",
                        selectInput(
                            "t3_projects_control",
                            "Projects:",
                            selected = "actix_full",
                            choice = unique(dfs$Project)
                        )
                    )
                ),
                fluidRow(
                    box(
                        title = "Codegen Units Optimization",
                        width = 12, solidHeader = TRUE, status = "primary",
                        plotlyOutput("t3_codegen_plot", height = "700")
                    )
                )
            ),
            tabItem(
                tabName = "raw",
                box(
                    title = "Raw Data Table", width = 12, solidHeader = TRUE, status = "primary",
                    dataTableOutput("t4_raw_table")
                )
            )
        )
    )
)

server <- function(input, output) {
  ## GENERAL TAB

  output$t0_general <- renderUI(HTML("
This benchmark is intended to compare how different combinations of HTTP Client and HTTP Server will affect the binary size of Rust applications.
These application we will be covering here has some basic requirements that are based on needs that we have for our real-world app.
More detail about them,
as well as the full source code to the apps and this dashboard,
is available at the <a href=\"https://github.com/OSSystems/rust-web-client-server-testing/\">project's GitHub page</a>.
<br><br>
We have analysed and present here variations not only to the full binary size,
but for the project's dependencies as well.
That information is extracted using the <a href=\"https://github.com/RazrFalcon/cargo-bloat/\">cargo-bloat</a>,
a community tool used to breaking down binary sizes of Rust applications.
<br><br>
The side menu here can be used to navigate between the different plots we have.
On the <b>Project Data</b> tab we show a plot comparing binary sizes across different projects as well as a breakdown of it's internal Crates sizes.
<b>Crates Data</b> presents a general view of Crates size distribution as well as a direct comparison of crates usage on all Projects.
<b>Optimization Data</b> tab shows how different optimization parameters affects the overall binary size.
Lastly <b>Full Data</b> has a relay large table with all the raw data we've extracted from these applications.
"))

  ## PROJECT TAB
  last_clicked <- reactiveVal(1)
  observe(if (input$menu == "projects") {
            observeEvent(event_data("plotly_click", source = "t1_project"), {
              clicked <- event_data("plotly_click", source = "t1_project")
              if (!is.null(clicked)) {
                ## When click the same X coord twice the X value would not change and other reactive
                ## contexts would not detect the sound click, however as the graph itself might actually
                ## change, a new click should still be processed, therefore we enforce the last_clicked
                ##  to always changed when clicked a valid coord
                last_clicked(-1)
                last_clicked(clicked$y)
              }
            })
          })

  df_crates <- reactive({
    if (is.null(last_clicked())) return(NULL)

    df <- isolate(df_projects())
    if (is.null(df)) return (NULL)
    projects <- sort(unique(df$Project))
    clicked <- last_clicked()
    clicked <- projects[clicked]
    df[
        df$Project == clicked &
        df$Optimization == input$t1_opt_control &
        df$LTO == input$t1_lto_control &
        df$Codegen.Units == input$t1_cu_control
       ,]
  })

  df_projects <- reactive({
    selected <- input$t1_projects_control
    if (length(selected) <= 0) return (NULL)
    dfs[
        dfs$Project %in% selected &
        dfs$Optimization == input$t1_opt_control &
        dfs$LTO == input$t1_lto_control &
        dfs$Codegen.Units == input$t1_cu_control
       ,]
  })

  output$t1_project_plot <- renderPlotly({
    df <- stack_df(df_projects(), input$t1_stack_control)
    if (is.null(df)) return (NULL)
    ggplotly(
        plot_projects(df) +
        aes(text = paste("Relative size: ", Relative.Size, "%", sep = "")) +
        theme(legend.title = element_blank()),
        source = "t1_project",
        layout.legend.y = 10000,
        tooltip = c("Size", "Crate", "text")
    ) %>%
      style(hoverinfo = "none", traces = 1) %>%
      config(displayModeBar = FALSE) %>%
      layout(
          legend = list(
              title = list(text="Crate")
          ),
          yaxis = list(fixedrange=TRUE),
          xaxis = list(fixedrange=TRUE)
      )
  })

  observe({
    output$t1_crates_plot <- renderPlotly({
      df <- df_crates()
      if (is.null(df)) return (NULL)
      ggplotly(
          plot_crates(df, df$Project[[1]]) +
          aes(text = paste("Relative size: ", Relative.Size, "%", sep = "")),
          height = nrow(df) * 24,
          tooltip = c("Size", "text")
      ) %>%
        style(hoverinfo = "none", traces = 1) %>%
        config(displayModeBar = FALSE) %>%
        layout(
            yaxis = list(fixedrange=TRUE),
            xaxis = list(fixedrange=TRUE)
        )
    })
  })

  ## CRATES TAB
  output$t2_box_plot <- renderPlotly({
    df <- dfs[
        dfs$Optimization == input$t2_opt_control &
        dfs$LTO == input$t2_lto_control &
        dfs$Codegen.Units == input$t2_cu_control
       ,]
    ggplotly(plot_crates_box(df)) %>%
      (function(ggly) {
        ggly$x$data[[1]]$text <- with(
            df,
            paste(
                "Project:", Project,
                "<br>Crate: ", Crate
            )
        )
        ggly$x$data[[1]]$hoverinfo <- c("box")
        ggly
      }) %>%
      config(displayModeBar = FALSE) %>%
      layout(
          yaxis = list(fixedrange=TRUE),
          xaxis = list(fixedrange=TRUE)
      )
  })

  output$t2_crates_plot <- renderPlotly({
    selected <- input$t2_crates_control
    if (length(selected) <= 0) return (NULL)
    df <- dfs[
        dfs$Crate %in% selected &
        dfs$Optimization == input$t2_opt_control &
        dfs$LTO == input$t2_lto_control &
        dfs$Codegen.Units == input$t2_cu_control
       ,]
    ggp <- plot_crates_dist(df, selected)
    if (is.null(ggp)) return (NULL)
    ggplotly(
        ggp +
        aes(text = paste("Relative size: ", Relative.Size, "%", sep = "")) +
        theme(legend.title = element_blank()),
        tooltip = c("Size", "text")
    ) %>%
      config(displayModeBar = FALSE) %>%
      layout(
          legend = list(
              title = list(text="Project")
          ),
          yaxis = list(fixedrange=TRUE),
          xaxis = list(fixedrange=TRUE)
      )
  })

  ## OPTIMIZATION TAB
  output$t3_optimization_plot <- renderPlotly({
    df <- dfs[
        dfs$Project %in% input$t3_projects_control
       ,]
    ggplotly(
        plot_optimization(df) +
        theme(legend.title = element_blank()),
        tooltip = c("Total")
    ) %>%
      config(displayModeBar = FALSE) %>%
      layout(
          legend = list(
              title = list(text="LTO, Codegen Units")
          ),
          yaxis = list(fixedrange=TRUE),
          xaxis = list(fixedrange=TRUE)
      )
  })

  output$t3_codegen_plot <- renderPlotly({
    df <- dfs[
        dfs$Optimization == "z" &
        dfs$LTO == "fat"
       ,]
    ggplotly(
        plot_codegen(df) +
        aes(text = paste("Optimization:", Optimization, "<br>LTO: ", LTO)) +
        theme(legend.title = element_blank()),
        tooltip = c("Total", "text")
    ) %>%
      (function(ggly) {
        ggly$x$data[[2]]$name <- "16"
        ggly$x$data[[2]]$legendgroup <- "16"
        ggly
      }) %>%
      config(displayModeBar = FALSE) %>%
      layout(
          legend = list(
              title = list(text = "Codegen Units"),
              values = list("1", "16")
          ),
          yaxis = list(fixedrange=TRUE),
          xaxis = list(fixedrange=TRUE)
      )
  })

  ## RAW TAB
  output$t4_raw_table <- renderDataTable(
      ## gen_table(dfs),
      dfs[, !(names(dfs) %in% c("Total", "Relative.Size"))],
      options = list(pageLength = 15)
  )
}

app = shinyApp(ui, server)
## options(browser = "echo")
## runApp(app, port = 8080)
