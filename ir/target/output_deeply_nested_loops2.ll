; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testSwappedWhileForLoops() {
entryID0:
  br label %while_condID1

while_condID1:                                    ; preds = %for_endID2, %entryID0
  br i1 true, label %while_bodyID1, label %while_endID1

while_bodyID1:                                    ; preds = %while_condID1
  br label %for_condID2

for_condID2:                                      ; preds = %for_incID2, %while_bodyID1
  br i1 true, label %for_bodyID2, label %for_endID2

for_bodyID2:                                      ; preds = %for_condID2
  br label %do_bodyID3

do_bodyID3:                                       ; preds = %do_condID3, %for_bodyID2
  br label %while_condID4

while_condID4:                                    ; preds = %for_endID5, %do_bodyID3
  br i1 true, label %while_bodyID4, label %while_endID4

while_bodyID4:                                    ; preds = %while_condID4
  br label %for_condID5

for_condID5:                                      ; preds = %for_incID5, %while_bodyID4
  br i1 true, label %for_bodyID5, label %for_endID5

for_bodyID5:                                      ; preds = %for_condID5
  br label %do_bodyID6

do_bodyID6:                                       ; preds = %do_condID6, %for_bodyID5
  ret i64 42
  br label %do_condID6

do_condID6:                                       ; preds = %do_bodyID6
  br i1 true, label %do_bodyID6, label %do_endID6

do_endID6:                                        ; preds = %do_condID6
  br label %for_incID5

for_incID5:                                       ; preds = %do_endID6
  br label %for_condID5

for_endID5:                                       ; preds = %for_condID5
  br label %while_condID4

while_endID4:                                     ; preds = %while_condID4
  br label %do_condID3

do_condID3:                                       ; preds = %while_endID4
  br i1 true, label %do_bodyID3, label %do_endID3

do_endID3:                                        ; preds = %do_condID3
  br label %for_incID2

for_incID2:                                       ; preds = %do_endID3
  br label %for_condID2

for_endID2:                                       ; preds = %for_condID2
  br label %while_condID1

while_endID1:                                     ; preds = %while_condID1
}
