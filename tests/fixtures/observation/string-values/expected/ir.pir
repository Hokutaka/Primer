; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %text@0: string = #1 "日本語\n\0":string
#2 %saved@1: string = #3 %text@0:string
#4 set %text@0:string = #5 "changed":string
#6 print.bool #7 eq.string(#8 %saved@1:string, #9 "日本語\n\0":string)
#10 print.string #11 %text@0:string
