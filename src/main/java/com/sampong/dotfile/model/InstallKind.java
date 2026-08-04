package com.sampong.dotfile.model;

/** Which flow triggered the streaming install popup — install vs remove (Phase 9), vs update
 *  a single already-installed package to its available version (Phase 13, net-new). */
public enum InstallKind { INSTALL, REMOVE, UPDATE }
