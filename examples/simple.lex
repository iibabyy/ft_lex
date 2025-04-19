%{
#include <memory.h>

void yyerror(char *message)
{
  printf("Error: %s\n",message);
}
%}

%x STRING

%%
a printf("Salut\n");

\" BEGIN(STRING);
<STRING>[^\\]\" BEGIN(INITIAL);

%%
AHAHA