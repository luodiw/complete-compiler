; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunction() {
entryID0:
  br i1 true, label %thenID1, label %elseID1

thenID1:                                          ; preds = %entryID0
  ret i64 1
  br label %mergeID1

elseID1:                                          ; preds = %entryID0
  ret i64 1
  br label %mergeID1

mergeID1:                                         ; preds = %elseID1, %thenID1
}
