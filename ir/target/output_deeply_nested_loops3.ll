; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testMultipleDoWhileLoops3() {
entryID0:
  br label %do_bodyID1

do_bodyID1:                                       ; preds = %do_condID1, %entryID0
  br label %do_bodyID2

do_bodyID2:                                       ; preds = %do_condID2, %do_bodyID1
  %i = alloca i64, align 8
  store i64 42, ptr %i, align 4
  br label %for_condID3

for_condID3:                                      ; preds = %for_incID3, %do_bodyID2
  br i1 true, label %for_bodyID3, label %for_endID3

for_bodyID3:                                      ; preds = %for_condID3
  br label %while_condID4

while_condID4:                                    ; preds = %do_endID5, %for_bodyID3
  br i1 true, label %while_bodyID4, label %while_endID4

while_bodyID4:                                    ; preds = %while_condID4
  br label %do_bodyID5

do_bodyID5:                                       ; preds = %do_condID5, %while_bodyID4
  ret i64 42
  br label %do_condID5

do_condID5:                                       ; preds = %do_bodyID5
  br i1 true, label %do_bodyID5, label %do_endID5

do_endID5:                                        ; preds = %do_condID5
  br label %while_condID4

while_endID4:                                     ; preds = %while_condID4
  br label %for_incID3

for_incID3:                                       ; preds = %while_endID4
  br label %for_condID3

for_endID3:                                       ; preds = %for_condID3
  br label %do_condID2

do_condID2:                                       ; preds = %for_endID3
  br i1 true, label %do_bodyID2, label %do_endID2

do_endID2:                                        ; preds = %do_condID2
  br label %do_condID1

do_condID1:                                       ; preds = %do_endID2
  br i1 true, label %do_bodyID1, label %do_endID1

do_endID1:                                        ; preds = %do_condID1
}
