#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Exercise only the synthetic Djinn host's observed native consent dialog.
Reads its process-owned window, captures it with PrintWindow, and clicks the
observed No/Yes button. Never operates another process's identity window.
"""
import ctypes as C, json, os, subprocess, time, hashlib
from ctypes import wintypes as W
from PIL import Image
import dramatis_review_receipts as R
assert os.name == "nt"
u=C.windll.user32;g=C.windll.gdi32
callback=C.WINFUNCTYPE(W.BOOL,W.HWND,W.LPARAM)
u.EnumWindows.argtypes=[callback,W.LPARAM];u.EnumChildWindows.argtypes=[W.HWND,callback,W.LPARAM]
u.GetWindowThreadProcessId.argtypes=[W.HWND,C.POINTER(W.DWORD)]
u.GetWindowTextW.argtypes=[W.HWND,W.LPWSTR,C.c_int]
u.GetDlgCtrlID.argtypes=[W.HWND];u.GetDlgCtrlID.restype=C.c_int
u.IsWindowVisible.argtypes=[W.HWND]
u.SendMessageW.argtypes=[W.HWND,W.UINT,C.c_size_t,C.c_ssize_t];u.SendMessageW.restype=C.c_ssize_t
u.GetWindowRect.argtypes=[W.HWND,C.POINTER(W.RECT)]
u.GetWindowDC.argtypes=[W.HWND];u.GetWindowDC.restype=W.HDC
u.ReleaseDC.argtypes=[W.HWND,W.HDC]
u.PrintWindow.argtypes=[W.HWND,W.HDC,W.UINT]
g.CreateCompatibleDC.argtypes=[W.HDC];g.CreateCompatibleDC.restype=W.HDC
g.CreateCompatibleBitmap.argtypes=[W.HDC,C.c_int,C.c_int];g.CreateCompatibleBitmap.restype=W.HBITMAP
g.SelectObject.argtypes=[W.HDC,W.HANDLE];g.SelectObject.restype=W.HANDLE
g.DeleteObject.argtypes=[W.HANDLE];g.DeleteDC.argtypes=[W.HDC]
class Header(C.Structure):
    _fields_=[("size",W.DWORD),("width",W.LONG),("height",W.LONG),("planes",W.WORD),("bits",W.WORD),("compression",W.DWORD),("image_size",W.DWORD),("x",W.LONG),("y",W.LONG),("used",W.DWORD),("important",W.DWORD)]
g.GetDIBits.argtypes=[W.HDC,W.HBITMAP,W.UINT,W.UINT,C.c_void_p,C.POINTER(Header),W.UINT]
def text(hwnd):
    s=C.create_unicode_buffer(4096);u.GetWindowTextW(hwnd,s,len(s));return s.value

def window(pid):
    found=[]
    @callback
    def visit(hwnd,_):
        owner=W.DWORD();u.GetWindowThreadProcessId(hwnd,C.byref(owner))
        if owner.value==pid and u.IsWindowVisible(hwnd) and text(hwnd)=="Djinn · Quarantine review": found.append(hwnd)
        return True
    u.EnumWindows(visit,0);return found[0] if found else None

def children(hwnd):
    found=[]
    @callback
    def visit(child,_):
        found.append((child,u.GetDlgCtrlID(child),text(child)));return True
    u.EnumChildWindows(hwnd,visit,0);return found

def capture(hwnd,path):
    rect=W.RECT();assert u.GetWindowRect(hwnd,C.byref(rect))
    width=rect.right-rect.left;height=rect.bottom-rect.top
    dc=u.GetWindowDC(hwnd);mem=g.CreateCompatibleDC(dc);bitmap=g.CreateCompatibleBitmap(dc,width,height);old=g.SelectObject(mem,bitmap)
    try:
        assert u.PrintWindow(hwnd,mem,2), "native dialog PrintWindow"
        g.SelectObject(mem,old)
        header=Header(40,width,-height,1,32,0,width*height*4,0,0,0,0)
        pixels=C.create_string_buffer(width*height*4)
        assert g.GetDIBits(mem,bitmap,0,height,pixels,C.byref(header),0)==height
        Image.frombytes("RGB",(width,height),pixels.raw,"raw","BGRX").save(path)
    finally:
        g.DeleteObject(bitmap);g.DeleteDC(mem);u.ReleaseDC(hwnd,dc)

def main():
    root=R.OUT/"runtime-native";assert not root.exists()
    endpoint=r"\\.\pipe\dramatis-app-native-consent"
    env=dict(os.environ,DRAMATIS_RECEIPT_ENDPOINT=endpoint)
    observed=[]
    flags=subprocess.CREATE_NO_WINDOW
    with (R.OUT/"native-host.log").open("w") as log:
        host=subprocess.Popen([str(R.TARGET/"debug/dramatis-review-host.exe"),endpoint,str(root),"native"],cwd=R.ROOT,stdout=log,stderr=subprocess.STDOUT,creationflags=flags)
        test=None
        try:
            deadline=time.monotonic()+60
            while not (root/"ready").exists():
                assert host.poll() is None and time.monotonic()<deadline;time.sleep(.1)
            for answer,button_id in [("deny",7),("approve",6)]:
                log_path=R.OUT/("native-"+answer+".log")
                with log_path.open("w") as out:
                    command=R.cargo("test","-p","dramatis-app","--test","state")+["--","--ignored","--exact","native_consent_receipt","--nocapture"]
                    test=subprocess.Popen(command,cwd=R.ROOT,env=dict(env,DRAMATIS_NATIVE_RESPONSE=answer),stdout=out,stderr=subprocess.STDOUT,creationflags=flags)
                    deadline=time.monotonic()+120
                    while True:
                        hwnd=window(host.pid)
                        parts=children(hwnd) if hwnd else []
                        joined="\n".join(t for _,_,t in parts)
                        if "Persona: family" in joined and "Personal payment card" in joined: break
                        assert test.poll() is None and time.monotonic()<deadline,"native consent window readiness"
                        time.sleep(.1)
                    time.sleep(.2)
                    assert window(host.pid)==hwnd,"only the synthetic resident's observed window"
                    capture(hwnd,R.OUT/("native-consent-"+answer+".png"))
                    target=next(child for child,id,label in parts if id==button_id)
                    observed.append({"answer":answer,"owner_process":"dramatis-review-host","title":text(hwnd),"contents":[t for _,_,t in parts],"button_id":button_id,"command":command})
                    u.SendMessageW(target,0x00F5,0,0) # BM_CLICK on the observed button
                    assert test.wait(timeout=60)==0,answer
                    test=None
        finally:
            if test is not None and test.poll() is None:
                subprocess.run(["taskkill","/PID",str(test.pid),"/T","/F"],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            if host.poll() is None:host.terminate()
            host.wait(timeout=30)
    R.remove_runtime(root)
    artifacts={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in R.OUT.glob("native-*") if p.is_file()}
    (R.OUT/"native-consent.json").write_text(json.dumps({"platform":"Windows native MessageBox","synthetic_only":True,"observed":observed,"artifacts_sha256":artifacts},indent=2)+"\n",encoding="utf-8")
    print("Native Djinn consent: denial preserved 3 cards; approval deleted only the selected card.")
if __name__=="__main__":main()
