const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const model = vm.createContext({});
vm.runInContext(fs.readFileSync(path.join(__dirname, '../ui/KeyboardModel.js'), 'utf8'), model);
const keys = rows => rows.flat().map(k => k[0]).filter(Boolean);

test('full size ISO has every physical key once and nonnegative spaces', () => {
    const info = {choice:'auto',id:'de',geometry:'iso',keyboard:{type:'external',fullSize:true}};
    const rows = model.buildRows(info);
    const names = keys(rows);
    assert.equal(names.length, 105);
    assert.equal(new Set(names).size, names.length);
    for (const name of ['kp0','kp+','kprt','ArrowUp','Home','IntlBackslash','ContextMenu']) assert.ok(names.includes(name));
    for (const row of rows) {
        assert.ok(row.every(k => k[2] > 0));
        assert.equal(row.reduce((sum,k) => sum+k[2],0),23);
    }
    assert.equal(model.isLaptop(info),false);
});
test('ANSI Enter is on home row and changing language cannot create a laptop', () => {
    const info = {choice:'us',id:'de',geometry:'iso',keyboard:{type:'external',fullSize:true}};
    const rows = model.buildRows(info);
    assert.equal(keys(rows).length,104);
    assert.ok(!rows[2].some(k => k[0] === 'Enter'));
    assert.ok(rows[3].some(k => k[0] === 'Enter'));
    assert.ok(!keys(rows).includes('IntlBackslash'));
    assert.equal(model.isIsoEnter(info),false);
    assert.equal(model.isLaptop(info),false);
});
test('TKL keeps navigation without adding a keypad, firmware-only leaves editor empty', () => {
    const info = {choice:'tkl',geometry:'iso',keyboard:{type:'external',fullSize:true}};
    const names = keys(model.buildRows(info));
    assert.equal(names.length,88);
    assert.ok(names.includes('ArrowDown'));
    assert.ok(!names.includes('kp0'));
    assert.equal(model.buildRows({available:false}).length,0);
});
test('ThinkPad remains a laptop in auto; manual geometry respects ISO/ANSI', () => {
    const info = {choice:'auto',id:'de',geometry:'iso',keyboard:{type:'built-in'}};
    const names = keys(model.buildRows(info));
    assert.ok(names.includes('fn'));
    assert.ok(names.includes('PageDown'));
    assert.ok(!names.includes('kp0'));
    assert.equal(model.isIsoEnter({...info, choice:'us'}),false);
    assert.equal(model.isIsoEnter({...info, choice:'de',geometry:'ansi'}),true);
});
