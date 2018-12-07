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
img_size=2048

sizeof_float=8

if len (sys.argv)<3:
    print("Usage:{0} <sid> <XY> [delay]".format(sys.argv[0]))
    sys.exit(0)

if len(sys.argv)>=4:
    delay=float(sys.argv[3])

mxr=np.zeros([img_size,img_size])
mxi=np.zeros([img_size,img_size])
wgt=np.zeros([img_size,img_size])
vis_xy=open(sys.argv[2],'rb')
#sid_file=open(sys.argv[1],'r')

for sid in open(sys.argv[1]):
    print sid.strip()
    xy = np.array(struct.unpack('<{0}d'.format(nchannels * 2 ), vis_xy.read(nchannels*2*sizeof_float)))
    xy=xy[::2]+1j*xy[1::2]
    cross_corr=xy
    sid_angle=float(sid)
    for i in range(0,nchannels):
        ch=ch_beg+i
        f=ch*freq_per_ch
        lbd=c/f
        cc=cross_corr[i]*np.exp(1j*delay/lbd*2*np.pi)
        u=bl*math.cos(sid_angle)/lbd
        v=bl*math.sin(sid_angle)/lbd
        ui=int(u/max_uv*(img_size/2)+img_size/2)
        vi=int(v/max_uv*(img_size/2)+img_size/2)
        if ui>=0 and ui<img_size and vi>=0 and vi<img_size and not math.isnan(cross_corr[i].real) and not math.isnan(cross_corr[i].imag):
            mxr[vi,ui]+=cc.real
            mxi[vi,ui]+=cc.imag
            wgt[vi,ui]+=1
          
print "fill uvmap"
        
for i in range(0,img_size):
    for j in range(0,img_size):
        if wgt[i,j]>0:
            mxr[i,j]/=wgt[i,j]
            mxi[i,j]/=wgt[i,j]
        if np.sqrt(mxr[i,j]**2+mxi[i,j]**2)>1500.0:
            mxr[i,j]=0
            mxi[i,j]=0



pyfits.PrimaryHDU(mxr).writeto('uvr.fits',clobber=True)
pyfits.PrimaryHDU(mxi).writeto('uvi.fits',clobber=True)

mx=mxr+1j*mxi
mx+=np.conj(mx[::-1, ::-1])
img=scipy.fftpack.fftshift(scipy.fftpack.fft2(scipy.fftpack.fftshift(mx))).real
pyfits.PrimaryHDU(img).writeto('img.fits',clobber=True)

