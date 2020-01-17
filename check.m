x=(-5:0.05:5)';

function [c]=ncdens(n,x)
  c=sqrt(log(log(n))/pi)*exp(-x.^2*log(log(n)));
end

function [c]=nccum(n,x)
  c=(erf(sqrt(log(log(n)))*x)+1)/2;
end

nc=nccum(n,x);
dnc=[diff(nc);0];

function [d]=d_totvar(xc,yc)
  d=sum(abs(xc-yc))/2;
end

## s=load("xw-m_10000-n_29.dat");
## m=10000;
## n=536870912;
## sc=histc(s,x)/m;

## s=load("clcg-m_10000-n_29.dat");
## m=10000;
## n=536870925;
## sc=histc(s,x)/m;

## s=load("homemade-m_1000-n_29.dat");
## m=1000;
## n=536870912;
## sc=histc(s,x)/m;

s=load("homemade-m_1000-n_29.dat");
m=1000;
n=536870912;
sc=histc(s,x)/m;

d_totvar(dnc,sc)

plot([dnc sc])
