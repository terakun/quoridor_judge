/*
qf_board_viewer.js
Copyright (c) 2014 Quoridor Fansite Webmaster
Released under the MIT license
http://opensource.org/licenses/mit-license.php
*/

var b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
var b64idxs = [];
var i;
for(i=0; i<b64chars.length; i++){
  b64idxs[b64chars.charAt(i)] = i;
}
var dMoves = [9, 10, 1, -8, -9, -10, -1, 8];
var wwbNames = ['qf_wwall', 'qf_bwall'];
var whvNames = ['qf_hwall', 'qf_vwall'];

function tMargin(bs){
  return parseInt(bs/3, 10);
}

function binToNum(bin, start, length){
  var i;
  var num = 0;
  for(i=0; i<length; i++){
    num += (bin[start+i] << (length-i-1));
  }
  return num;
}

function base64Decode(qfObj, qfCode){
  var i, j, len, num, p, pieceP, tTurn;
  var bin = [];

  len = qfCode.length;
  for(i=0; i<len; i++){
    num = b64idxs[qfCode.charAt(i)];
    if(num == null) continue;

    for(j=0; j<6; j++){
      bin[6*i+j] = (num >>> (5-j)) & 1;
    }
  }

  p = 2;
  qfObj.state = {};
  if(bin[0]){
    qfObj.hasState = true;

    qfObj.state.piece = [];
    qfObj.state.piece[0] = binToNum(bin, p, 7);
    p += 7;
    qfObj.state.piece[1] = binToNum(bin, p, 7);
    p += 7;

    qfObj.state.wallNum = [];
    qfObj.state.walls = [];
    for(i=0; i<4; i++){
      len = qfObj.state.wallNum[i] = binToNum(bin, p, 4);
      p += 4;
      qfObj.state.walls[i] = [];
      for(j=0; j<len; j++){
        qfObj.state.walls[i][j] = binToNum(bin, p, 6);
        p += 6;
      }
    }

    qfObj.state.lastmove = [];
    tTurn = qfObj.state.lastmove[0] = bin[p];
    p += 1;
    num = qfObj.state.lastmove[1] = bin[p];
    p += 1;
    if(num){
      qfObj.state.lastmove[2] = binToNum(bin, p, 6);
      p += 6;
    }

    qfObj.state.turnNum = binToNum(bin, p, 10);
    p += 10;
    qfObj.state.turn = 1-qfObj.state.lastmove[0];
  } else {
    qfObj.hasState = false;

    qfObj.state.piece = [4, 76];
    qfObj.state.wallNum = [0, 0, 0, 0];
    qfObj.state.turnNum = 1;
    tTurn = 1;
    qfObj.state.turn = 0;
  }

  if(bin[1]){
    qfObj.hasRecord = true;
    qfObj.record = {};

    len = qfObj.record.moveNum = binToNum(bin, p, 10);
    p += 10;

    pieceP = [];
    pieceP[0] = qfObj.state.piece[0];
    pieceP[1] = qfObj.state.piece[1];
    qfObj.record.moves = [];
    for(i=0; i<len; i++){
      tTurn = 1-tTurn;

      qfObj.record.moves[i] = [];
      qfObj.record.moves[i][0] = bin[p];
      p += 1;

      if(!qfObj.record.moves[i][0]){
        //piece
        num = binToNum(bin, p, 3);
        p += 3;

        qfObj.record.moves[i][1] = pieceP[tTurn];
        pieceP[tTurn] += dMoves[num];
        if(pieceP[tTurn] === pieceP[1-tTurn]){
          pieceP[tTurn] += dMoves[num];
        }
        qfObj.record.moves[i][2] = pieceP[tTurn];
      } else {
        //wall
        qfObj.record.moves[i][1] = bin[p];
        p += 1;
        qfObj.record.moves[i][2] = binToNum(bin, p, 6);
        p += 6;
      }
    }

    qfObj.record.current = 0;
    qfObj.record.currentW = 0;
  } else {
    qfObj.hasRecord = false;
  }
  /*
  qfObj properties are made by this function
    qfObj.hasState -> false or true
    qfObj.hasRecord -> false or true

    qfObj.state.piece[0] -> white piece place
    qfObj.state.piece[1] -> black piece place

    qfObj.state.wallNum[0] -> whwall
    qfObj.state.wallNum[1] -> wvwall
    qfObj.state.wallNum[2] -> bhwall
    qfObj.state.wallNum[3] -> bvwall

    qfObj.state.walls[0...3][0...wallmaxnum] -> wall place

    qfObj.state.lastmove[0] -> turn. white: 0, black: 1
    qfObj.state.lastmove[1] -> pw. piece: 0, wall: 1
    qfObj.state.lastmove[2] -> place(if wall)

    qfObj.state.turnNum -> turn num
    qfObj.state.turn -> turn. white: 0, black: 1

    qfObj.record.moveNum -> move num

    qfObj.record.moves[0...movenum][0] -> pw. piece: 0, wall: 1
    qfObj.record.moves[0...movenum][1] -> if piece: backPlace, if wall: hv
    qfObj.record.moves[0...movenum][2] -> place

    qfObj.record.current -> current index
    qfObj.record.currentW -> current wall index
    */
}

function createWholeBoard(idx, el){
  var i;
  var t = $(el);
  var bs = parseInt(t.data('boardsize'), 10);
  if(isNaN(bs)){
    bs = 10;
  }
  var fl = parseInt(t.data('flags'), 10);
  if(isNaN(fl)){
    fl = 5;
  }
  var tCode = t.data('qfcode');
  var qfCode;
  if(tCode){
    qfCode = String(tCode);
  } else {
    qfCode = 'gkwAAAAQ';
  }

  var fls = [];
  for(i=0; i<4; i++){
    fls[i] = (fl >>> i) & 1;
  }
  //fls[0]->infobar, fls[1]->Gray, fls[2]->LM, fls[3]->inv

  var qfObj = {};
  base64Decode(qfObj, qfCode);

  createAllHTML(el, bs, fls, qfObj);
  if(qfObj.hasRecord){
    adjustButtonEvents(el, bs, fls, qfObj);
  }
}

function createAllHTML(el, bs, fls, qfObj){	
  var i, j, wb, hv;
  var t = $(el);

  var grids = '';
  for(i=0; i<81; i++){
    grids += '<div class="qf_board_grid" style="width: '+(4*bs)+'px; height: '+(4*bs)+'px; top: '+(5*bs*(8-parseInt(i/9, 10)))+'px; left: '+(5*bs*(i%9))+'px;"></div>';
  }

  var walls = '';
  for(i=0; i<4; i++){
    wb = parseInt(i/2, 10);
    hv = i%2;
    for(j=0; j<qfObj.state.wallNum[i]; j++){
      walls += '<div class="qf_wall '+(wwbNames[wb])+' '+(whvNames[hv])+'" style="width: '+(bs+8*bs*(1-hv))+'px; height: '+(bs+8*bs*hv)+'px; top: '+(5*bs*(7-parseInt(qfObj.state.walls[i][j]/8, 10))+4*bs*(1-hv))+'px; left: '+(5*bs*(qfObj.state.walls[i][j]%8)+4*bs*hv)+'px;"></div>';
    }
  }

  var moveWalls = '';
  var tTurn = qfObj.state.turn;
  var ctrlButton = '';
  if(qfObj.hasRecord){
    for(i=0; i<qfObj.record.moveNum; i++){
      if(qfObj.record.moves[i][0]){
        hv = qfObj.record.moves[i][1];
        moveWalls += '<div class="qf_wall '+(wwbNames[tTurn])+' '+(whvNames[hv])+' qf_mwall" style="width: '+(bs+8*bs*(1-hv))+'px; height: '+(bs+8*bs*hv)+'px; top: '+(5*bs*(7-parseInt(qfObj.record.moves[i][2]/8, 10))+4*bs*(1-hv))+'px; left: '+(5*bs*(qfObj.record.moves[i][2]%8)+4*bs*hv)+'px; opacity: 0; -webkit-transform: scale(3); -moz-transform: scale(3); -ms-transform: scale(3); -o-transform: scale(3); transform: scale(3);"></div>';
      }
      tTurn = 1-tTurn;
    }

    ctrlButton = '\
      <div class="qf_controlpanel" style="width: '+(8*bs)+'px; height: '+(11*bs)+'px; top: '+(0)+'px; left: '+(50*bs)+'px;">\
      <button class="qf_control_button qf_b_back" style="width: '+(8*bs)+'px; height: '+(4*bs)+'px; top: '+(2*bs)+'px; left: '+(0)+'px; font-size: '+(parseInt((5*bs)/3, 10))+'px;">BACK</button>\
      <button class="qf_control_button qf_b_next" style="width: '+(8*bs)+'px; height: '+(4*bs)+'px; top: '+(7*bs)+'px; left: '+(0)+'px; font-size: '+(parseInt((5*bs)/3, 10))+'px;">NEXT</button>\
      <button class="qf_control_button qf_cb_cover qf_b_back_c" style="width: '+(8*bs)+'px; height: '+(4*bs)+'px; top: '+(2*bs)+'px; left: '+(0)+'px; font-size: '+(parseInt((5*bs)/3, 10))+'px;">BACK</button>\
      <button class="qf_control_button qf_cb_cover qf_b_next_c" style="display: none; width: '+(8*bs)+'px; height: '+(4*bs)+'px; top: '+(7*bs)+'px; left: '+(0)+'px; font-size: '+(parseInt((5*bs)/3, 10))+'px;">NEXT</button>\
      </div>\
    ';
  }

  var infoBar = '';
  if(fls[0]){
    infoBar = '\
      <div class="qf_infobar" style="width: '+(48*bs)+'px; height: '+(5*bs)+'px; top: '+(48*bs)+'px; left: '+(0)+'px; border-bottom-width: '+(parseInt(bs/3, 10))+'px;">\
      <div class="qf_info_text qf_info_turn" style="width: '+(6*bs)+'px; height: '+(3*bs)+'px; top: '+(34)+'%; left: '+(4*bs)+'px; font-size: '+(2*bs)+'px;">TURN</div>\
      <div class="qf_info_text_num qf_info_turn_num" style="width: '+(6*bs)+'px; height: '+(4*bs)+'px; top: '+(bs)+'px; left: '+(10*bs)+'px; font-size: '+(3*bs)+'px;">'+(qfObj.state.turnNum)+'</div>\
      <div class="qf_info_text qf_info_white" style="width: '+(8*bs)+'px; height: '+(3*bs)+'px; top: '+(34)+'%; left: '+(18*bs)+'px; font-size: '+(2*bs)+'px;">WHITE</div>\
      <div class="qf_info_text_num qf_info_white_num" style="width: '+(5*bs)+'px; height: '+(4*bs)+'px; top: '+(bs)+'px; left: '+(26*bs)+'px; font-size: '+(3*bs)+'px;">'+(10-(qfObj.state.wallNum[0]+qfObj.state.wallNum[1]))+'</div>\
      <div class="qf_info_text qf_info_black" style="width: '+(8*bs)+'px; height: '+(3*bs)+'px; top: '+(34)+'%; left: '+(32*bs)+'px; font-size: '+(2*bs)+'px;">BLACK</div>\
      <div class="qf_info_text_num qf_info_black_num" style="width: '+(5*bs)+'px; height: '+(4*bs)+'px; top: '+(bs)+'px; left: '+(40*bs)+'px; font-size: '+(3*bs)+'px;">'+(10-(qfObj.state.wallNum[2]+qfObj.state.wallNum[3]))+'</div>\
      </div>\
    ';
  }

  var allTags = '\
    <div class="qf_gameboard" style="width: '+(48*bs)+'px; height: '+(48*bs)+'px;">\
    <div class ="qf_inner_gameboard" style="width: '+(44*bs)+'px; height: '+(44*bs)+'px; top: '+(2*bs)+'px; left: '+(2*bs)+'px;">\
  '+ grids +'\
    <div class="qf_piece qf_piece_white" style="width: '+(4*bs-2*tMargin(bs))+'px; height: '+(4*bs-2*tMargin(bs))+'px; top: '+(5*bs*(8-parseInt(qfObj.state.piece[0]/9, 10))+tMargin(bs))+'px; left: '+(5*bs*(qfObj.state.piece[0]%9)+tMargin(bs))+'px;"></div>\
    <div class="qf_piece qf_piece_black" style="width: '+(4*bs-2*tMargin(bs))+'px; height: '+(4*bs-2*tMargin(bs))+'px; top: '+(5*bs*(8-parseInt(qfObj.state.piece[1]/9, 10))+tMargin(bs))+'px; left: '+(5*bs*(qfObj.state.piece[1]%9)+tMargin(bs))+'px;"></div>\
  '+ walls + moveWalls +'\
    </div>\
    </div>\
  '+ infoBar + ctrlButton +'\
  ';

  var hR;
  if(qfObj.hasRecord){
    hR = 1;
  } else {
    hR = 0;
  }
  t.css({
    width: 48*bs+10*bs*hR,
    height: 48*bs+6*bs*fls[0]
  }).append(allTags);

  //wall gray
  if(fls[1]){
    t.find('.qf_wall').addClass('qf_gwall');
  }

  //lastmove highlight
  var count = 0;
  var targetLM;
  if((fls[2]) && (qfObj.hasState)){
    if(qfObj.state.lastmove[1]){
      //wall
      loop: for(i=0; i<4; i++){
        for(j=0; j<qfObj.state.wallNum[i]; j++){
          if(qfObj.state.walls[i][j] === qfObj.state.lastmove[2]){
            break loop;
          } else {
            count++;
          }
        }
      }
      targetLM = t.find('.qf_wall').eq(count);
    } else {
      //piece
      targetLM = t.find('.qf_piece').eq(qfObj.state.lastmove[0]);
    }
    targetLM.addClass('qf_first_lm');
    highlightOn(targetLM, bs);
  }

  //inverse
  if(fls[3]){
    t.find('.qf_gameboard').addClass('qf_boardInversed');
  }
}

function highlightOn(obj, bs){
  var length = parseInt(bs/6, 10);
  if(length === 0){
    length = 1;
  }

  if(obj.hasClass('qf_piece')) {
    obj.css({
      width: 4*bs-2*tMargin(bs)-2*length,
      height: 4*bs-2*tMargin(bs)-2*length
    });
  } else if(obj.hasClass('qf_hwall')){
    obj.css({
      width: 9*bs-2*length,
      height: bs-2*length
    });
  } else if(obj.hasClass('qf_vwall')){
    obj.css({
      width: bs-2*length,
      height: 9*bs-2*length
    });
  }

  obj.css({
    borderStyle: 'solid',
    borderWidth: length
  }).addClass('qf_highlighted');
}

function highlightOff(obj, bs){
  if(obj.hasClass('qf_piece')) {
    obj.css({
      width: 4*bs-2*tMargin(bs),
      height: 4*bs-2*tMargin(bs)
    });
  } else if(obj.hasClass('qf_hwall')){
    obj.css({
      width: 9*bs,
      height: bs
    });
  } else if(obj.hasClass('qf_vwall')){
    obj.css({
      width: bs,
      height: 9*bs
    });
  }

  obj.css({
    borderStyle: 'none',
  }).removeClass('qf_highlighted');
}


function draw_qf_board() {
  function adjustButtonEvents(el, bs, fls, qfObj){
    var t = $(el);

    t.find('.qf_b_back').on('click', function(){
      var target;

      //remove highlight
      if(fls[2]){
        target = t.find('.qf_highlighted');
        highlightOff(target, bs);
      }

      //move
      qfObj.record.current -= 1;
      qfObj.state.turn = 1-qfObj.state.turn;
      if(qfObj.record.moves[qfObj.record.current][0]){
        //wall
        qfObj.record.currentW -= 1;
        target = t.find('.qf_mwall').eq(qfObj.record.currentW);

        //infobar
        if(fls[0]){
          if(target.hasClass('qf_wwall')){
            qfObj.state.wallNum[0] -= 1;
            t.find('.qf_info_white_num').html(10-(qfObj.state.wallNum[0]+qfObj.state.wallNum[1]));
          } else {
            qfObj.state.wallNum[2] -= 1;
            t.find('.qf_info_black_num').html(10-(qfObj.state.wallNum[2]+qfObj.state.wallNum[3]));
          }
        }

        target.css({
          'opacity': '0',
          '-webkit-transform': 'scale(3)',
          '-moz-transform': 'scale(3)',
          '-ms-transform': 'scale(3)',
          '-o-transform': 'scale(3)',
          'transform': 'scale(3)'
        });
      } else {
        //piece
        target = t.find('.qf_piece').eq(qfObj.state.turn);
        target.css({
          top: 5*bs*(8-parseInt(qfObj.record.moves[qfObj.record.current][1]/9, 10)) + tMargin(bs) +'px',
          left: 5*bs*(qfObj.record.moves[qfObj.record.current][1]%9) + tMargin(bs) +'px'
        });
      }

      //highlight
      if(fls[2] && (!((qfObj.record.current === 0) && (!qfObj.hasState)))){
        if(qfObj.record.current){
          if(qfObj.record.moves[qfObj.record.current-1][0]){
            //wall
            target = t.find('.qf_mwall').eq(qfObj.record.currentW-1);
          } else {
            //piece
            target = t.find('.qf_piece').eq(1-qfObj.state.turn);
          }
        } else {
          //first turn
          target = t.find('.qf_first_lm');
        }
        highlightOn(target, bs);
      }

      //infobar
      if(fls[0]){
        qfObj.state.turnNum -= 1;
        t.find('.qf_info_turn_num').html(qfObj.state.turnNum);
      }

      if(qfObj.record.current === 0){
        t.find('.qf_b_back_c').css('display', 'block');
      }
      t.find('.qf_b_next_c').css('display', 'none');
    });

    t.find('.qf_b_next').on('click', function(){
      var target;

      //remove highlight
      if((fls[2]) && (!((qfObj.record.current === 0) && (!qfObj.hasState)))){
        target = t.find('.qf_highlighted');
        highlightOff(target, bs);
      }

      //move
      if(qfObj.record.moves[qfObj.record.current][0]){
        //wall
        target = t.find('.qf_mwall').eq(qfObj.record.currentW);

        //infobar
        if(fls[0]){
          if(target.hasClass('qf_wwall')){
            qfObj.state.wallNum[0] += 1;
            t.find('.qf_info_white_num').html(10-(qfObj.state.wallNum[0]+qfObj.state.wallNum[1]));
          } else {
            qfObj.state.wallNum[2] += 1;
            t.find('.qf_info_black_num').html(10-(qfObj.state.wallNum[2]+qfObj.state.wallNum[3]));
          }
        }

        target.css({
          'opacity': '1',
          '-webkit-transform': 'scale(1)',
          '-moz-transform': 'scale(1)',
          '-ms-transform': 'scale(1)',
          '-o-transform': 'scale(1)',
          'transform': 'scale(1)'
        });

        qfObj.record.currentW += 1;
      } else {
        //piece
        target = t.find('.qf_piece').eq(qfObj.state.turn);
        target.css({
          top: 5*bs*(8-parseInt(qfObj.record.moves[qfObj.record.current][2]/9, 10)) + tMargin(bs) +'px',
          left: 5*bs*(qfObj.record.moves[qfObj.record.current][2]%9) + tMargin(bs) +'px'
        });
      }

      //highlight
      if(fls[2]){
        highlightOn(target, bs);
      }

      //infobar
      if(fls[0]){
        qfObj.state.turnNum += 1;
        t.find('.qf_info_turn_num').html(qfObj.state.turnNum);
      }

      qfObj.record.current += 1;
      qfObj.state.turn = 1-qfObj.state.turn;

      if(qfObj.record.current === qfObj.record.moveNum){
        t.find('.qf_b_next_c').css('display', 'block');
      }
      t.find('.qf_b_back_c').css('display', 'none');
    });
  }

  //execute part
  $('.qf_board_viewer').each(createWholeBoard);
}

$(draw_qf_board);
