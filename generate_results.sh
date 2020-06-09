#! /bin/bash

declare WORKSPACE_MEMBERS=(
  "dummy"
  "actix_full"
  "actix_reqwest"
  "gotham_reqwest"
  "hyper_full"
  "hyper_reqwest"
  "warp_surf"
  "tide_surf"
  "warp_reqwest"
)

declare OPTIMISATION_FLAGS=(
  "0"
  "1"
  "2"
  "3"
  "s"
  "z"
)

declare LTO_FLAGS=(
  "thin"
  "fat"
)

declare CODEGEN_UNITS=(
  "1"
  "16"
)

for flag in ${OPTIMISATION_FLAGS[@]}; do
  for member in ${WORKSPACE_MEMBERS[@]}; do
    for lto in ${LTO_FLAGS[@]}; do
      for units in ${CODEGEN_UNITS[@]}; do
        printf '[profile.release]
opt-level = %s
lto = "%s"
codegen-units = %i
'\
               "$([[ $flag =~ [0-3] ]] && echo $flag || echo "\"$flag\"")"\
               "$lto"\
               "$units"\
               > .cargo/config
        cargo bloat -p $member --release \
              --crates -n 200 --message-format=json \
              > shiny_app/results/${member}_opt_${flag}_lto_${lto}_cg_$units
      done
    done
  done
done

rm .cargo/config
