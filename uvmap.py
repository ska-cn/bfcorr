#!/usr/bin/env python

import struct
import sys
import pylab
import math
import cmath
import numpy
import astropy.io.fits as pyfits
import scipy.fftpack
import config

ch_beg=config.ch_beg;
ch_end=config.ch_end;
ch_max=2048
nchannels=ch_end-ch_beg


min_f=50E6
max_f=200E6;

freq_per_ch=250e6/2048

delay=0*1.47
#delay=0
max_bl=2740;
bl=2740
#bl=2740
c=2.99792458E8
max_uv=max_bl/(c/max_f)
img_size=2048

sizeof_float=8

if len (sys.argv)<5:
    print("Usage:{0} <sid> <XY> <XX> <YY> [delay]".format(sys.argv[0]))
    sys.exit(0)


if len(sys.argv)==6:
    delay=float(sys.argv[5])
        
mxr=numpy.zeros([img_size,img_size])
mxi=numpy.zeros([img_size,img_size])
wgt=numpy.zeros([img_size,img_size])
vis_xy=open(sys.argv[2],'rb')
vis_xx=open(sys.argv[3],'rb')
vis_yy=open(sys.argv[4],'rb')
sid_file=open(sys.argv[1],'r')

for sid in sid_file:
    print sid
    xx=struct.unpack('<{0}d'.format(nchannels*2),vis_xx.read(nchannels*2*sizeof_float))
    yy=struct.unpack('<{0}d'.format(nchannels*2),vis_yy.read(nchannels*2*sizeof_float))
    xy=struct.unpack('<{0}d'.format(nchannels*2),vis_xy.read(nchannels*2*sizeof_float))
    xy_c=[xy[i*2]+xy[i*2+1]*1j for i in range(0,nchannels)]
    xx_c=[xx[i*2] for i in range(0,nchannels)]
    yy_c=[yy[i*2] for i in range(0,nchannels)]
    cross_corr=[xy_c[i]/math.sqrt(xx_c[i]*yy_c[i])*cmath.exp(1j*delay/(c/((i+ch_beg)*freq_per_ch))*2*math.pi) for i in range(0,nchannels)]
    sid_angle=float(sid)
    for i in range(0,nchannels):
        ch=ch_beg+i
        f=ch*freq_per_ch
        l=c/f
        u=bl*math.cos(sid_angle)/l
        v=bl*math.sin(sid_angle)/l
        ui=int(u/max_uv*(img_size/2)+img_size/2)
        vi=int(v/max_uv*(img_size/2)+img_size/2)
        if ui>=0 and ui<img_size and vi>=0 and vi<img_size and not math.isnan(cross_corr[i].real) and not math.isnan(cross_corr[i].imag):
            mxr[vi,ui]+=cross_corr[i].real
            mxi[vi,ui]+=cross_corr[i].imag
            wgt[vi,ui]+=1
            
print "uvmap filled"    
        
for i in range(0,img_size):
    for j in range(0,img_size):
        if wgt[i,j]>0:
            mxr[i,j]/=wgt[i,j]
            mxi[i,j]/=wgt[i,j]
        if abs(mxr[i,j])>.05:
            mxr[i,j]=0
            mxi[i,j]=0

pyfits.PrimaryHDU(mxr).writeto('r.fits',clobber=True)
pyfits.PrimaryHDU(mxi).writeto('i.fits',clobber=True)
mxr-=mxr.mean()
mxi-=mxi.mean()


mx=mxr+1j*mxi
img=scipy.fftpack.fftshift(scipy.fftpack.fft2(scipy.fftpack.fftshift(mx))).real
pyfits.PrimaryHDU(img).writeto('img.fits',clobber=True)
