XOPTS=()
XFLAGS=()
if [[ "${STABLE:-0}"  != 0 ]]; then
  XOPTS+=( "+1.31.0" )
else
  XFLAGS+=( "--features" "nightly" )
fi

switchup() {
  cargo update -p $1 -Z minimal-versions -Z no-index-update --precise $2
}

switchlist() {
  local name=$1
  shift
  while true; do
    local this=$1
    local next=$2
    if [ "${this}" == "" ]; then
      break
    fi
    if [ "${next}" == "" ]; then
      break
    fi
    switchup "${name}:${this}" "${next}"
    shift
  done
}

cargo update -Z no-index-update
cargo update -Z minimal-versions -Z no-index-update

# This file contains hacks required to get various cargo build targets working
# on modern rust.

advanced() {
  #  switchlist libc  0.1.{0..5} 0.1.10 0.1.12 \
  #                 0.2.{0..7}
  # num 0.1.27 is the earliest that compiles on rust 1.31.0
  switchlist time 0.1.0 0.1.25

  switchlist num 0.1.0 0.1.27
  switchlist gcc 0.1.0 0.3.0 0.3.4
  switchlist libc 0.1.0 0.1.5
  # gcc 0.3.4 is the earliest that compiles on rust 1.31.0
  true;
}

simple() {
  #switchup time:0.1.0 0.1.25
  #switchup gcc:0.3.0 0.3.4
  #switchup libc:0.1.0 0.1.5
  if [[ "${STABLE:-0}" != 0 ]]; then
    switchup ccache_stats_reader:0.1.0 0.1.2
  else
    switchup chrono:0.2.24 0.4.0
    switchup num:0.1.0 0.1.27
  fi
  true;
}

#advanced
simple



cargo "${XOPTS[@]}" build "${XFLAGS[@]}" --all-targets &&
  cargo test
