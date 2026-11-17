// The same SplitMix64 streams and PDG tree-level matrix elements as scattering.rs.
// FP64 throughout; no fast-math. All event fields are real resident data, not padding.
typedef unsigned long long U64;
__device__ U64 next(U64 &state) {
    U64 z = (state += 0x9e3779b97f4a7c15ULL);
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}
__device__ double uniform(U64 &state) {
    return ((double)(next(state) >> 12) + 0.5) / 4503599627370496.0;
}
__device__ double matrix_element(int p, double s, double t, double u, double alpha) {
    double s2=s*s, t2=t*t, u2=u*u;
    double g=4.0*3.14159265358979323846*alpha;
    double a;
    switch(p) {
    case 0: a=2.0*(t2+u2)/s2; break;
    case 1: a=4.0/9.0*(s2+u2)/t2; break;
    case 2: a=4.0/9.0*((s2+u2)/t2+(s2+t2)/u2)-8.0/27.0*s2/(u*t); break;
    case 3: a=4.0/9.0*(t2+u2)/s2; break;
    case 4: a=4.0/9.0*((s2+u2)/t2+(t2+u2)/s2)-8.0/27.0*u2/(s*t); break;
    case 5: a=(s2+u2)/t2-4.0/9.0*(s2+u2)/(s*u); break;
    case 6: a=32.0/27.0*(u2+t2)/(u*t)-8.0/3.0*(u2+t2)/s2; break;
    case 7: a=1.0/6.0*(u2+t2)/(u*t)-3.0/8.0*(u2+t2)/s2; break;
    default: a=4.5*(3.0-u*t/s2-u*s/t2-s*t/u2); break;
    }
    return g*g*a;
}
extern "C" __global__ void generate(double *events, U64 n, U64 offset, U64 seed,
                                     int process, double energy, double alpha, double cut) {
    U64 stride=(U64)blockDim.x*gridDim.x;
    for(U64 i=(U64)blockIdx.x*blockDim.x+threadIdx.x; i<n; i+=stride) {
        U64 state=seed ^ ((offset+i)*0xd2b74407b1ce6e93ULL);
        state=next(state);
        double c=cut*(2.0*uniform(state)-1.0);
        double phi=6.28318530717958647692*uniform(state);
        double e=energy/2.0, r=sqrt(1.0-c*c), s=energy*energy;
        double m2=matrix_element(process,s,-s*(1.0-c)/2.0,-s*(1.0+c)/2.0,alpha);
        double symmetry=(process==2 || process==6 || process==8)?0.5:1.0;
        double weight=2.0*cut*symmetry*m2/(32.0*3.14159265358979323846*s)*389379372.1;
        events[0*n+i]=c;
        events[1*n+i]=phi;
        events[2*n+i]=e;
        events[3*n+i]=e*r*cos(phi);
        events[4*n+i]=e*r*sin(phi);
        events[5*n+i]=e*c;
        events[6*n+i]=weight;
        events[7*n+i]=m2;
    }
}
// Each block accumulates a bounded shared histogram. All resident events contribute.
extern "C" __global__ void reduce(const double *events, double *partials, U64 n, double cut) {
    __shared__ double h[122];
    for(int j=threadIdx.x;j<122;j+=blockDim.x)h[j]=0.0;
    __syncthreads();
    U64 stride=(U64)blockDim.x*gridDim.x;
    for(U64 i=(U64)blockIdx.x*blockDim.x+threadIdx.x;i<n;i+=stride) {
        int bin=min(59,(int)((events[i]/cut+1.0)*30.0));
        double w=events[6*n+i];
        atomicAdd(h+bin,w);
        atomicAdd(h+60+bin,w*w);
        atomicAdd(h+120,events[3*n+i]);
        atomicAdd(h+121,events[4*n+i]);
    }
    __syncthreads();
    for(int j=threadIdx.x;j<122;j+=blockDim.x)partials[(U64)blockIdx.x*122+j]=h[j];
}
extern "C" __global__ void gather(const double *events, double *sample, U64 n, U64 count) {
    U64 j=(U64)blockIdx.x*blockDim.x+threadIdx.x;
    if(j<count) {
        U64 i=j*n/count;
        for(int field=0;field<8;++field)sample[j*8+field]=events[(U64)field*n+i];
    }
}
