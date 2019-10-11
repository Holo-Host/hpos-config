
const { Config } = require('..')
const { expect } = require('chai')

var assert = require('assert');
describe('Array', function() {
    describe('#indexOf()', function() {
	it('should return -1 when the value is not present', function() {
	    assert.equal([1, 2, 3].indexOf(4), -1);
	});
    });
});


describe('Config', function() {
    describe('constructor', function() {
	it('should construct', function() {
	    const config = new Config('a@b.ca', 'password', undefined, false)
	});
    });
});
