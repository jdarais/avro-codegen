export default {
  spec_dir: "spec",
  spec_files: [
    "**/*[sS]pec.?(m)[tj]s"
  ],
  helpers: [
    "helpers/**/*.?(m)[tj]s"
  ],
  env: {
    stopSpecOnExpectationFailure: false,
    random: true,
    forbidDuplicateNames: true
  }
}
