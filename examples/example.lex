%{
  #include <stdio.h>
  #include <stdlib.h>
%}

%{
  int num_lines = 0;
  int foo = 12;
%}

%p 2500
%n 500
%a 2
%e 1000
%k 1000
%o 3000

%array

%s COMMENT FOO
%x STRING TEST

DIGIT       [0-9]    
ALPHA      [A-Za-z]
ID         {ALPHA}({ALPHA}|{DIGIT})*
HEX        \\x[0-9A-Fa-f]{1,2}
%%


"b"         { BEGIN(TEST); }
"s"         { BEGIN(INITIAL); }

^m$a printf("what ?!\n");

\n printf("newline\n");

. printf("peu importe...\n");				

a/b |printf("peu importe...\n");		


<TEST>a* printf("what ?!\n");

<COMMENT>{
  {sdfsdf}a        { BEGIN(INITIAL); }
}

<STRING>{
  \"          { BEGIN(INITIAL); }
  {HEX}       { printf("HEX ESCAPE: %s\n", yytext); }
  \\n         { printf("NEWLINE ESCAPE\n"); }
  \\\"        { printf("QUOTE ESCAPE\n"); }
  \n          { fprintf(stderr, "ERROR: Newline in string\n"); exit(1); }
  .           { printf("STRING CHAR: %s\n", yytext); }
}

%%
int main() {
  yylex();
  printf("Lines: %d\n", num_lines);
  return 0;
}

int yywrap() {
  return 1;
}