#!/usr/bin/env python
import matplotlib.pylab as plt
import numpy as np
import sys
data=np.loadtxt(sys.argv[1])
if data.shape[1]==2:
    plt.plot(data[:,0],data[:,1],'r')
elif data.shape[1]==3:
    plt.plot(data[:,0],data[:,1],'k')
    plt.plot(data[:,0],data[:,2],'r')
plt.savefig('spec.png')
plt.show()

