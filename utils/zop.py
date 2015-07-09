#!/usr/bin/env python2

last_colour = ""

def colour(char):
    global last_colour
    cmd = ""
    if char == '\'':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 2, background: 2},\nZOP::PrintOps{text: "'
    if char == '>' or char == '-':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 3, background: 3},\nZOP::PrintOps{text: "'
    if char == '+':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 4, background: 4},\nZOP::PrintOps{text: "'
    if char == '&':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 5, background: 5},\nZOP::PrintOps{text: "'
    if char == ',' or  char == ';':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 6, background: 6},\nZOP::PrintOps{text: "'
    if char == '$' or char == '%':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 7, background: 7},\nZOP::PrintOps{text: "'
    if char == "o" or char == '=':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 8, background: 8},\nZOP::PrintOps{text: "'
    if char == '.' or char == '@':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 9, background: 9},\nZOP::PrintOps{text: "'
    if char == '*':
        cmd += '".to_string()},\nZOP::SetColor{foreground: 10, background: 10},\nZOP::PrintOps{text: "'
    if cmd == last_colour:
      return ""
    else:
      if cmd != "":
        last_colour = cmd
      return cmd

def readFiles(fileName):
    global last_colour
    last_colour = ""
    #nc = "start\nset_text_style 10\n"
    nc = "ZOP::SetTextStyle{bold: true, reverse: false, monospace: true, italic: false},\n"
    with open(fileName) as f:
        content = f.readlines()

    del content[63]
    del content[62]
    del content[61]
    del content[60]
    del content[59]
    del content[58]
    del content[57]
    del content[56]
    del content[55]
    del content[54]
    del content[53]
    del content[52]
    del content[51]
    del content[50]
    del content[49]
    del content[48]
    del content[47]

    del content[19]
    del content[18]
    del content[17]
    del content[16]
    del content[15]
    del content[14]
    del content[13]
    del content[12]
    del content[11]
    del content[10]
    del content[9]
    del content[8]
    del content[7]
    del content[6]
    del content[5]
    del content[4]
    del content[3]
    del content[2]
    del content[1]
    del content[0]
    #content.remove(content[63])
    #content.remove(content[60])
    #content.remove(content[59])
    #content.remove(content[58])

    #content.remove(content[15])
    #content.remove(content[17])
    #content.remove(content[16])
    #content.remove(content[15])
    #content.remove(content[14])
    #content.remove(content[13])
    #content.remove(content[12])
    #content.remove(content[11])
    #content.remove(content[4])
    #content.remove(content[10])
    ##content.remove(content[10])
    #content.remove(content[13])
    #content.remove(content[13])
    #content.remove(content[13])
    #content.remove(content[15])
    #content.remove(content[15])
    #content.remove(content[15])
    #content.remove(content[9])
    #content.remove(content[7])
    #content.remove(content[7])
    #content.remove(content[6])
    #content.remove(content[7])
    #content.remove(content[8])

    #content.remove(content[10])
    #content.remove(content[11])
    #content.remove(content[12])
    #content.remove(content[13])
    #content.remove(content[14])

    #last = content[0][2]
    nc += 'ZOP::PrintOps{text: "'
    last = '0'
    for line in content:
        line = line.replace("#", "o");
        line = line.replace("\"", "");
        line = line.replace("\",", "");
        nc += colour(line[0])
        #nc += "print "
        for char in line:
            if char == last:
                nc += (char if char != '\'' else '\\\'') if char != "\n" else ""
            else:
                nc += colour(char)
                nc += (char if char != '\'' else '\\\'') if char != "\n" else ""
            last = char
        nc += '".to_string()},\nZOP::Newline,\nZOP::PrintOps{text: "'

    #nc += "\nquit\n"
    nc += "\n"
    print nc.replace('ZOP::PrintOps{text: "\n', '')


for num in range(1,13):
    #print num
    print 'ZOP::Routine{name: "nyanpr' + str(num) + '".to_string(), count_variables: 1},'
    #print "routine pr1 0\n"
    #readFiles("frames/frame1.txt")
    readFiles("frames/frame{}.txt".format(num))
    print "ZOP::Ret{value: Operand::new_const(0)},"


#print "label p4\n"
#readFiles("frames/frame3.txt")
#print "ret 0\n"

#print "label p5\n"
#readFiles("frames/frame4.txt")
#print "ret 0\n"
