#!/usr/bin/env python

import struct
import sys
import math
import cmath
import numpy as np
import astropy.io.fits as pyfits
import scipy.fftpack
from scipy.optimize import fmin
ch_beg=400;
ch_end=1640;
ch_max=2048
nchannels=ch_end-ch_beg


min_f=50E6
max_f=200E6;

freq_per_ch=250e6/2048

delay=0*1.47
#delay=0
max_bl=229-89;
bl=229-89
#bl=2740
c=2.99792458E8
max_uv=max_bl/(c/max_f)


sizeof_float=8

if len (sys.argv)<4:
    print("Usage:{0} uvr uvi delay".format(sys.argv[0]))
    sys.exit(0)


delay=float(sys.argv[3])
        

mxr=pyfits.open(sys.argv[1])[0].data
mxi=pyfits.open(sys.argv[2])[0].data

img_size=mxr.shape[0]
mx=mxr+1j*mxi
mx+=np.conj(mx[::-1, ::-1])
img=scipy.fftpack.fftshift(scipy.fftpack.fft2(scipy.fftpack.fftshift(mx))).real
pyfits.PrimaryHDU(img).writeto('img.fits',clobber=True)


l=-500
def cal(delay):
    mxr1=np.zeros([img_size,img_size])
    mxi1=np.zeros([img_size,img_size])
    for i in range(0, img_size):
        for j in range(0, img_size):
            if mxr[i,j]==0 and mxi[i,j]==0:
                continue
            vis=mxr[i,j]+1j*mxi[i,j]
            r=((i-img_size/2)**2+(j-img_size/2)**2)**0.5
            uv=float(r)/(img_size/2)*max_uv
            if uv==0:
                continue
            lbd=bl/uv;

            if lbd<=0:
                continue
            vis_cal=vis*np.exp(1j*delay/lbd*2*np.pi)
            mxr1[i,j]=vis_cal.real
            mxi1[i,j]=vis_cal.imag
    mx=mxr1+1j*mxi1
    mx+=np.conj(mx[::-1, ::-1])
    img=scipy.fftpack.fftshift(scipy.fftpack.fft2(scipy.fftpack.fftshift(mx))).real
    pyfits.PrimaryHDU(mxr1).writeto('r.fits',clobber=True)
    pyfits.PrimaryHDU(mxi1).writeto('i.fits',clobber=True)
    pyfits.PrimaryHDU(img).writeto('img.fits',clobber=True)

    #img[img_size/2-50:img_size/2+50, img_size/2-50:img_size/2+50]=0
    result= -img.max()
    print delay,result
    return result


print cal(delay)
#sys.exit(0)

result=fmin(cal,delay)
print result
cal(result[0])

sys.exit(0)
min_v=1e99
opt_delay=0
delay=-600
f=open('result.txt','w')
while delay>-2000:
    v=cal(delay)
    f.write("{0} {1}\n".format(delay, v))
    f.flush()
    if v<min_v:
        min_v=v
        opt_delay=delay
    print delay, v, opt_delay, min_v
    delay-=1
