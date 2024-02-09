#include<bits/stdc++.h>
using namespace std;
#define FOR(i,a,b) for(int i=a;i<=b;++i)
int a [2000];
int f(int x){
	return f(x*2);
}
int main(){
	FOR(i,1,1900000){
		a[i]++;
		a[i]*=2;
		a[i]+=a[i-1];
	}
	return 0;
}
