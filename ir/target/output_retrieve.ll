; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunctionWithRetrieve() {
entryID0:
  %test_var = alloca i64, align 8
  store i64 0, ptr %test_var, align 4
  %vrecallID1 = load i64, ptr %test_var, align 4
  %test_var_2 = alloca i64, align 8
  store i64 %vrecallID1, ptr %test_var_2, align 4
}
