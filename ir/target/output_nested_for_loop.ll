; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testForLoopNested() {
entryID0:
  %test_var_outer = alloca i64, align 8
  store i64 0, ptr %test_var_outer, align 4
  br label %for_condID1

for_condID1:                                      ; preds = %for_incID1, %entryID0
  br i1 true, label %for_bodyID1, label %for_endID1

for_bodyID1:                                      ; preds = %for_condID1
  %test_var = alloca i64, align 8
  store i64 0, ptr %test_var, align 4
  br label %for_condID2

for_condID2:                                      ; preds = %for_incID2, %for_bodyID1
  br i1 true, label %for_bodyID2, label %for_endID2

for_bodyID2:                                      ; preds = %for_condID2
  br label %for_incID2
  br label %for_incID2

for_incID2:                                       ; preds = %for_bodyID2, %for_bodyID2
  store i64 42, ptr %test_var, align 4
  br label %for_condID2

for_endID2:                                       ; preds = %for_condID2
  br label %for_incID1

for_incID1:                                       ; preds = %for_endID2
  store i64 42, ptr %test_var_outer, align 4
  br label %for_condID1

for_endID1:                                       ; preds = %for_condID1
}
