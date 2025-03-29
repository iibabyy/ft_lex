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
%a 2000
%e 1000
%k 1000
%o 3000

%array

%s COMMENT
%x STRING

DIGIT      [0-9]
ALPHA      [A-Za-z]
ID         {ALPHA}({ALPHA}|{DIGIT})*
HEX        \\x[0-9A-Fa-f]{1,2}
%%

<INITIAL>{
  "/*"         { BEGIN(COMMENT); }
  \"           { BEGIN(STRING); }
  {ID}         { printf("ID: %s\n", yytext); }
  {DIGIT}+     { printf("NUMBER: %s\n", yytext); }
  \n           { num_lines++; }
  [ \t]+       
  .            { printf("Unknown: %s\n", yytext); }
}

<COMMENT>{
  "*/"        { BEGIN(INITIAL); }
  \n          { num_lines++; }
  .           
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